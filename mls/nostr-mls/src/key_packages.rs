//! Nostr MLS Key Packages

use nostr::util::hex;
use nostr::{Event, Kind, PublicKey, RelayUrl, Tag, TagKind};
use nostr_mls_storage::NostrMlsStorageProvider;
use openmls::key_packages::KeyPackage;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_traits::storage::StorageProvider;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use crate::error::Error;
use crate::NostrMls;

impl<Storage> NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    /// Creates a key package for a Nostr event.
    ///
    /// This function generates a hex-encoded key package that is used as the content field of a kind:443 Nostr event.
    /// The key package contains the user's credential and capabilities required for MLS operations.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A hex-encoded string containing the serialized key package
    /// * A tuple of tags for the Nostr event
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * It fails to generate the credential and signature keypair
    /// * It fails to build the key package
    /// * It fails to serialize the key package
    pub fn create_key_package_for_event<I>(
        &self,
        public_key: &PublicKey,
        relays: I,
    ) -> Result<(String, [Tag; 4]), Error>
    where
        I: IntoIterator<Item = RelayUrl>,
    {
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

        let key_package_serialized = key_package_bundle.key_package().tls_serialize_detached()?;

        let tags = [
            Tag::custom(TagKind::MlsProtocolVersion, ["1.0"]),
            Tag::custom(
                TagKind::MlsCiphersuite,
                [self.ciphersuite_value().to_string()],
            ),
            Tag::custom(TagKind::MlsExtensions, [self.extensions_value()]),
            Tag::relays(relays),
        ];

        Ok((hex::encode(key_package_serialized), tags))
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
    fn parse_serialized_key_package(&self, key_package_hex: &str) -> Result<KeyPackage, Error> {
        let key_package_bytes = hex::decode(key_package_hex)?;

        let key_package_in = KeyPackageIn::tls_deserialize(&mut key_package_bytes.as_slice())?;

        // Validate the signature, ciphersuite, and extensions of the key package
        let key_package =
            key_package_in.validate(self.provider.crypto(), ProtocolVersion::Mls10)?;

        Ok(key_package)
    }

    /// Parse key package from [`Event`]
    pub fn parse_key_package(&self, event: &Event) -> Result<KeyPackage, Error> {
        if event.kind != Kind::MlsKeyPackage {
            return Err(Error::UnexpectedEvent {
                expected: Kind::MlsKeyPackage,
                received: event.kind,
            });
        }

        self.parse_serialized_key_package(&event.content)
    }

    /// Deletes a key package from the MLS provider's storage.
    /// TODO: Do we need to delete the encryption keys from the MLS storage provider?
    ///
    /// # Arguments
    ///
    /// * `key_package` - The key package to delete
    pub fn delete_key_package_from_storage(&self, key_package: &KeyPackage) -> Result<(), Error> {
        let hash_ref = key_package.hash_ref(self.provider.crypto())?;

        self.provider
            .storage()
            .delete_key_package(&hash_ref)
            .map_err(|e| Error::Provider(e.to_string()))?;

        Ok(())
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
    pub(crate) fn generate_credential_with_key(
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
        let relays = vec![RelayUrl::parse("wss://relay.example.com").unwrap()];

        // Create key package
        let (key_package_hex, tags) = nostr_mls
            .create_key_package_for_event(&test_pubkey, relays.clone())
            .expect("Failed to create key package");

        // Create new instance for parsing
        let parsing_mls = create_test_nostr_mls();

        // Parse and validate the key package
        let key_package = parsing_mls
            .parse_serialized_key_package(&key_package_hex)
            .expect("Failed to parse key package");

        // Verify the key package has the expected properties
        assert_eq!(key_package.ciphersuite(), DEFAULT_CIPHERSUITE);

        assert_eq!(tags.len(), 4);
        assert_eq!(tags[0].kind(), TagKind::MlsProtocolVersion);
        assert_eq!(tags[1].kind(), TagKind::MlsCiphersuite);
        assert_eq!(tags[2].kind(), TagKind::MlsExtensions);
        assert_eq!(tags[3].kind(), TagKind::Relays);

        assert_eq!(
            tags[3].content().unwrap(),
            relays
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    #[test]
    fn test_key_package_deletion() {
        let nostr_mls = create_test_nostr_mls();
        let test_pubkey =
            PublicKey::from_hex("884704bd421671e01c13f854d2ce23ce2a5bfe9562f4f297ad2bc921ba30c3a6")
                .unwrap();

        let relays = vec![RelayUrl::parse("wss://relay.example.com").unwrap()];

        // Create and parse key package
        let (key_package_hex, _) = nostr_mls
            .create_key_package_for_event(&test_pubkey, relays.clone())
            .expect("Failed to create key package");

        // Create new instance for parsing and deletion
        let deletion_mls = create_test_nostr_mls();
        let key_package = deletion_mls
            .parse_serialized_key_package(&key_package_hex)
            .expect("Failed to parse key package");

        // Delete the key package
        deletion_mls
            .delete_key_package_from_storage(&key_package)
            .expect("Failed to delete key package");
    }

    #[test]
    fn test_invalid_key_package_parsing() {
        let nostr_mls = create_test_nostr_mls();

        // Try to parse invalid hex
        let result = nostr_mls.parse_serialized_key_package("invalid hex");
        assert!(matches!(result, Err(Error::Hex(..))));

        // Try to parse valid hex but invalid key package
        let result = nostr_mls.parse_serialized_key_package("deadbeef");
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
