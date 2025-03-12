//! Nostr MLS Key Packages

use nostr::util::hex;
use nostr::{Event, EventBuilder, Kind, NostrSigner, PublicKey, RelayUrl, Tag, TagKind};
use openmls::key_packages::KeyPackage;
use openmls::prelude::*;
use openmls::storage::StorageProvider;
use openmls_basic_credential::SignatureKeyPair;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use crate::error::Error;
use crate::NostrMls;

impl<Storage> NostrMls<Storage>
where
    Storage: StorageProvider,
{
    /// Creates a key package for a Nostr event.
    ///
    /// This function generates a key package that can be used in a Nostr event to join an MLS group.
    /// The key package contains the user's credential and capabilities required for MLS operations.
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
    pub fn create_key_package_for_event(&self, public_key: &PublicKey) -> Result<String, Error> {
        let (credential, signature_keypair) = self.generate_credential_with_key(public_key)?;

        let capabilities: Capabilities = self.capabilities();

        let key_package_bundle = KeyPackage::builder()
            .leaf_node_capabilities(capabilities)
            .mark_as_last_resort()
            .build(
                self.ciphersuite,
                &self.provider,
                &signature_keypair,
                credential,
            )?;

        // serialize the key package, then encode it to hex and put it in the content field
        let key_package_serialized = key_package_bundle.key_package().tls_serialize_detached()?;

        Ok(hex::encode(key_package_serialized))
    }

    /// Create key package [`Event`]
    ///
    /// The output [`Event`] is ready to be sent to the relays of the receiver public key.
    pub async fn create_key_package<T, I>(
        &self,
        signer: &T,
        receiver: &PublicKey,
        relays: I,
    ) -> Result<Event, Error>
    where
        T: NostrSigner,
        I: IntoIterator<Item = RelayUrl>,
    {
        let serialized_key_package: String = self.create_key_package_for_event(receiver)?;

        let ciphersuite: String = self.ciphersuite_value().to_string();
        let extensions: String = self.extensions_value();

        let tags = [
            Tag::custom(TagKind::MlsProtocolVersion, ["1.0"]),
            Tag::custom(TagKind::MlsCiphersuite, [ciphersuite]),
            Tag::custom(TagKind::MlsExtensions, [extensions]),
            Tag::relays(relays),
            Tag::protected(),
        ];

        let builder: EventBuilder =
            EventBuilder::new(Kind::MlsKeyPackage, serialized_key_package).tags(tags);

        Ok(builder.sign(signer).await?)
    }

    /// Parses and validates a hex-encoded key package.
    ///
    /// This function takes a hex-encoded key package string, decodes it, deserializes it into a
    /// KeyPackageIn object, and validates its signature, ciphersuite, and extensions.
    ///
    /// # Arguments
    ///
    /// * `key_package_hex` - A hex-encoded string containing the serialized key package
    ///
    /// # Returns
    ///
    /// A validated KeyPackage on success, or a Error on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The hex decoding fails
    /// * The TLS deserialization fails
    /// * The key package validation fails (invalid signature, ciphersuite, or extensions)
    pub fn parse_key_package(&self, key_package_hex: &str) -> Result<KeyPackage, Error> {
        let key_package_bytes = hex::decode(key_package_hex)?;

        let key_package_in = KeyPackageIn::tls_deserialize(&mut key_package_bytes.as_slice())?;

        // Validate the signature, ciphersuite, and extensions of the key package
        let key_package =
            key_package_in.validate(self.provider.crypto(), ProtocolVersion::Mls10)?;

        Ok(key_package)
    }

    /// Parse key package from [`Event`]
    pub fn parse_key_package_event(&self, event: &Event) -> Result<KeyPackage, Error> {
        if event.kind != Kind::MlsKeyPackage {
            return Err(Error::UnexpectedEvent {
                expected: Kind::MlsKeyPackage,
                received: event.kind,
            });
        }

        self.parse_key_package(&event.content)
    }

    /// Deletes a key package from the MLS provider's storage.
    ///
    /// This function deletes the key package from the MLS provider's storage.
    ///
    /// # Arguments
    ///
    /// * `key_package` - The key package to delete
    pub fn delete_key_package_from_storage(&self, key_package: KeyPackage) -> Result<(), Error> {
        let hash_ref = key_package.hash_ref(self.provider.crypto())?;

        self.provider
            .storage()
            .delete_key_package(&hash_ref)
            .map_err(|e| Error::Provider(e.to_string()))
    }

    /// Generates a credential with a key for MLS (Messaging Layer Security) operations.
    ///
    /// This function creates a new credential and associated signature key pair for use in MLS.
    /// It uses the default NostrMls configuration and stores the generated key pair in the
    /// crypto provider's storage.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The user's nostr pubkey
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
        &self,
        public_key: &PublicKey,
    ) -> Result<(CredentialWithKey, SignatureKeyPair), Error> {
        // Encode to hex
        let public_key: String = public_key.to_hex();

        let credential = BasicCredential::new(public_key.into());
        let signature_keypair = SignatureKeyPair::new(self.ciphersuite.signature_algorithm())?;

        signature_keypair
            .store(self.provider.storage())
            .map_err(|e| Error::Provider(e.to_string()))?;

        Ok((
            CredentialWithKey {
                credential: credential.into(),
                signature_key: signature_keypair.public().into(),
            },
            signature_keypair,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::DEFAULT_CIPHERSUITE;
    use crate::tests::create_test_nostr_mls;

    #[test]
    fn test_key_package_creation_and_parsing() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            PublicKey::from_hex("884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6")
                .unwrap();

        // Create key package
        let key_package_hex = nostr_mls
            .create_key_package_for_event(&test_pubkey)
            .expect("Failed to create key package");

        // Create new instance for parsing
        let parsing_mls = create_test_nostr_mls();

        // Parse and validate the key package
        let key_package = parsing_mls
            .parse_key_package(&key_package_hex)
            .expect("Failed to parse key package");

        // Verify the key package has the expected properties
        assert_eq!(key_package.ciphersuite(), DEFAULT_CIPHERSUITE);
    }

    #[test]
    fn test_key_package_deletion() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            PublicKey::from_hex("884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6")
                .unwrap();

        // Create and parse key package
        let key_package_hex = nostr_mls
            .create_key_package_for_event(&test_pubkey)
            .expect("Failed to create key package");

        // Create new instance for parsing and deletion
        let deletion_mls = create_test_nostr_mls();
        let key_package = deletion_mls
            .parse_key_package(&key_package_hex)
            .expect("Failed to parse key package");

        // Delete the key package
        deletion_mls
            .delete_key_package_from_storage(key_package)
            .expect("Failed to delete key package");
    }

    #[test]
    fn test_invalid_key_package_parsing() {
        let nostr_mls = create_test_nostr_mls();

        // Try to parse invalid hex
        let result = nostr_mls.parse_key_package("invalid hex");
        assert!(matches!(result, Err(Error::Hex(..))));

        // Try to parse valid hex but invalid key package
        let result = nostr_mls.parse_key_package("deadbeef");
        assert!(matches!(result, Err(Error::Tls(..))));
    }

    #[test]
    fn test_credential_generation() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            PublicKey::from_hex("884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6")
                .unwrap();

        let result = nostr_mls.generate_credential_with_key(&test_pubkey);
        assert!(result.is_ok());
    }
}
