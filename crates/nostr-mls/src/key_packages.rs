// TODO: Re-enable this once we've worked out whether we really need it.
// use crate::nostr_credential::{NostrCredential, SignatureKeyPair};
// Re-export KeyPackage from openmls for consumers
pub use openmls::key_packages::KeyPackage;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_traits::storage::StorageProvider;
use thiserror::Error;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use crate::NostrMls;

#[derive(Debug, Error, Eq, PartialEq, Clone)]
pub enum KeyPackageError {
    #[error("Error generating a signature keypair: {0}")]
    SignatureKeypairError(String),

    #[error("Error storing the signature keypair: {0}")]
    StoreSignatureKeypairError(String),

    #[error("Error generating the key package: {0}")]
    KeyPackageError(String),

    #[error("Error serializing the key package: {0}")]
    KeyPackageSerializationError(String),

    #[error("Error parsing the key package: {0}")]
    KeyPackageParseError(String),

    #[error("Invalid key package: {0}")]
    InvalidKeyPackage(String),

    #[error("Could not delete key package: {0}")]
    CouldNotDeleteKeyPackage(String),
}

/// Creates a key package for a Nostr event.
///
/// This function generates a key package that can be used in a Nostr event to join an MLS group.
/// The key package contains the user's credential and capabilities required for MLS operations.
///
/// # Arguments
///
/// * `pubkey` - The hex-encoded string of the user's nostr pubkey
/// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
///
/// # Returns
///
/// A hex-encoded string containing the serialized key package on success, or a KeyPackageError on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * It fails to generate the credential and signature keypair
/// * It fails to build the key package
/// * It fails to serialize the key package
pub fn create_key_package_for_event(
    pubkey: String,
    nostr_mls: &NostrMls,
) -> Result<String, KeyPackageError> {
    let (credential, signature_keypair) = generate_credential_with_key(pubkey, nostr_mls)?;

    let capabilities: Capabilities = Capabilities::new(
        None,
        Some(&[nostr_mls.ciphersuite]),
        Some(NostrMls::REQUIRED_EXTENSIONS),
        None,
        None,
    );

    let key_package_bundle = KeyPackage::builder()
        .leaf_node_capabilities(capabilities)
        .mark_as_last_resort()
        .build(
            nostr_mls.ciphersuite,
            &nostr_mls.provider,
            &signature_keypair,
            credential,
        )
        .map_err(|e| KeyPackageError::KeyPackageError(e.to_string()))?;

    // serialize the key package, then encode it to hex and put it in the content field
    let key_package_serialized = key_package_bundle
        .key_package()
        .tls_serialize_detached()
        .map_err(|e| KeyPackageError::KeyPackageSerializationError(e.to_string()))?;

    Ok(hex::encode(key_package_serialized))
}

/// Parses and validates a hex-encoded key package.
///
/// This function takes a hex-encoded key package string, decodes it, deserializes it into a
/// KeyPackageIn object, and validates its signature, ciphersuite, and extensions.
///
/// # Arguments
///
/// * `key_package_hex` - A hex-encoded string containing the serialized key package
/// * `nostr_mls` - A reference to the NostrMls instance containing the crypto provider
///
/// # Returns
///
/// A validated KeyPackage on success, or a KeyPackageError on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * The hex decoding fails
/// * The TLS deserialization fails
/// * The key package validation fails (invalid signature, ciphersuite, or extensions)
pub fn parse_key_package(
    key_package_hex: String,
    nostr_mls: &NostrMls,
) -> Result<KeyPackage, KeyPackageError> {
    let key_package_bytes = hex::decode(key_package_hex)
        .map_err(|e| KeyPackageError::KeyPackageParseError(e.to_string()))?;

    let key_package_in = KeyPackageIn::tls_deserialize(&mut key_package_bytes.as_slice())
        .map_err(|e| KeyPackageError::KeyPackageParseError(e.to_string()))?;

    // Validate the signature, ciphersuite, and extensions of the key package
    let key_package = key_package_in
        .validate(nostr_mls.provider.crypto(), ProtocolVersion::Mls10)
        .map_err(|e| KeyPackageError::InvalidKeyPackage(e.to_string()))?;

    Ok(key_package)
}

/// Deletes a key package from the MLS provider's storage.
///
/// This function deletes the key package from the MLS provider's storage.
///
/// # Arguments
///
/// * `key_package` - The key package to delete
/// * `nostr_mls` - The NostrMls instance containing the MLS provider
pub fn delete_key_package_from_storage(
    key_package: KeyPackage,
    nostr_mls: &NostrMls,
) -> Result<(), KeyPackageError> {
    let hash_ref = key_package
        .hash_ref(nostr_mls.provider.crypto())
        .map_err(|e| KeyPackageError::CouldNotDeleteKeyPackage(e.to_string()))?;

    nostr_mls
        .provider
        .storage()
        .delete_key_package(&hash_ref)
        .map_err(|e| KeyPackageError::CouldNotDeleteKeyPackage(e.to_string()))
}

/// Generates a credential with a key for MLS (Messaging Layer Security) operations.
///
/// This function creates a new credential and associated signature key pair for use in MLS.
/// It uses the default NostrMls configuration and stores the generated key pair in the
/// crypto provider's storage.
///
/// # Arguments
///
/// * `pubkey` - The hex-encoded string of the user's nostr pubkey
///
/// # Returns
///
/// A tuple containing:
/// * `CredentialWithKey` - The generated credential along with its public key.
/// * `SignatureKeyPair` - The generated signature key pair.
///
/// # Errors
///
/// This function will return an error if:
/// * It fails to generate a signature key pair.
/// * It fails to store the signature key pair in the crypto provider's storage.
pub fn generate_credential_with_key(
    pubkey: String,
    nostr_mls: &NostrMls,
) -> Result<(CredentialWithKey, SignatureKeyPair), KeyPackageError> {
    let credential = BasicCredential::new(pubkey.clone().into());
    let signature_keypair = SignatureKeyPair::new(nostr_mls.ciphersuite.signature_algorithm())
        .map_err(|e| KeyPackageError::SignatureKeypairError(e.to_string()))?;

    tracing::debug!("BasicCredential keypair generated for {:?}", pubkey);

    signature_keypair
        .store(nostr_mls.provider.storage())
        .map_err(|e| KeyPackageError::StoreSignatureKeypairError(e.to_string()))?;

    Ok((
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keypair.public().into(),
        },
        signature_keypair,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_nostr_mls;
    #[test]
    fn test_key_package_creation_and_parsing() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            "884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6".to_string();

        // Create key package
        let key_package_hex = create_key_package_for_event(test_pubkey, &nostr_mls)
            .expect("Failed to create key package");

        // Create new instance for parsing
        let parsing_mls = create_test_nostr_mls();

        // Parse and validate the key package
        let key_package =
            parse_key_package(key_package_hex, &parsing_mls).expect("Failed to parse key package");

        // Verify the key package has the expected properties
        assert_eq!(key_package.ciphersuite(), NostrMls::DEFAULT_CIPHERSUITE);
    }

    #[test]
    fn test_key_package_deletion() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            "884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6".to_string();

        // Create and parse key package
        let key_package_hex = create_key_package_for_event(test_pubkey, &nostr_mls)
            .expect("Failed to create key package");

        // Create new instance for parsing and deletion
        let deletion_mls = create_test_nostr_mls();
        let key_package =
            parse_key_package(key_package_hex, &deletion_mls).expect("Failed to parse key package");

        // Delete the key package
        delete_key_package_from_storage(key_package, &deletion_mls)
            .expect("Failed to delete key package");
    }

    #[test]
    fn test_invalid_key_package_parsing() {
        let nostr_mls = create_test_nostr_mls();

        // Try to parse invalid hex
        let result = parse_key_package("invalid hex".to_string(), &nostr_mls);
        assert!(matches!(
            result,
            Err(KeyPackageError::KeyPackageParseError(_))
        ));

        // Try to parse valid hex but invalid key package
        let result = parse_key_package("deadbeef".to_string(), &nostr_mls);
        assert!(matches!(
            result,
            Err(KeyPackageError::KeyPackageParseError(_))
        ));
    }

    #[test]
    fn test_credential_generation() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            "884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6".to_string();

        let result = generate_credential_with_key(test_pubkey.clone(), &nostr_mls);
        assert!(result.is_ok());
    }
}
