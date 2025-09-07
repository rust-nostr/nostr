// Copyright (c) 2024-2025 Jeff Gardner
// Copyright (c) 2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Group Extension functionality for MLS Group Context.
//! This is a required extension for Nostr Groups as per NIP-104.

use std::collections::BTreeSet;
use std::str;

use nostr::secp256k1::rand::rngs::OsRng;
use nostr::secp256k1::rand::Rng;
use nostr::{PublicKey, RelayUrl};
use openmls::extensions::{Extension, ExtensionType};
use openmls::group::{GroupContext, MlsGroup};
use tls_codec::{
    DeserializeBytes, TlsDeserialize, TlsDeserializeBytes, TlsSerialize, TlsSerializeBytes, TlsSize,
};

use crate::constant::NOSTR_GROUP_DATA_EXTENSION_TYPE;
use crate::error::Error;

/// TLS-serializable representation of Nostr Group Data Extension.
///
/// This struct is used exclusively for TLS codec serialization/deserialization
/// when the extension is transmitted over the MLS protocol. It uses `Vec<u8>`
/// for optional binary fields to allow empty vectors to represent `None` values,
/// which avoids the serialization issues that would occur with fixed-size arrays.
///
/// Users should not interact with this struct directly - use `NostrGroupDataExtension`
/// instead, which provides proper type safety and a clean API.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    TlsSerialize,
    TlsDeserialize,
    TlsDeserializeBytes,
    TlsSerializeBytes,
    TlsSize,
)]
pub(crate) struct TlsNostrGroupDataExtension {
    pub nostr_group_id: [u8; 32],
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub admin_pubkeys: Vec<Vec<u8>>,
    pub relays: Vec<Vec<u8>>,
    pub image_hash: Vec<u8>,  // Use Vec<u8> to allow empty for None
    pub image_key: Vec<u8>,   // Use Vec<u8> to allow empty for None
    pub image_nonce: Vec<u8>, // Use Vec<u8> to allow empty for None
}

/// This is an MLS Group Context extension used to store the group's name,
/// description, ID, admin identities, image URL, and image encryption key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NostrGroupDataExtension {
    /// Nostr Group ID
    pub nostr_group_id: [u8; 32],
    /// Group name
    pub name: String,
    /// Group description
    pub description: String,
    /// Group admins
    pub admins: BTreeSet<PublicKey>,
    /// Relays
    pub relays: BTreeSet<RelayUrl>,
    /// Group image hash, assuming the app will use single Blossom server
    pub image_hash: Option<[u8; 32]>,
    /// Private key to decrypt group image (encrypted when stored)
    pub image_key: Option<[u8; 32]>,
    /// Nonce to decrypt group image
    pub image_nonce: Option<[u8; 12]>,
}

impl NostrGroupDataExtension {
    /// Nostr Group Data extension type
    pub const EXTENSION_TYPE: u16 = NOSTR_GROUP_DATA_EXTENSION_TYPE;

    /// Creates a new NostrGroupDataExtension with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group
    /// * `description` - A description of the group's purpose
    /// * `admin_identities` - A list of Nostr public keys that have admin privileges
    /// * `relays` - A list of relay URLs where group messages will be published
    ///
    /// # Returns
    ///
    /// A new NostrGroupDataExtension instance with a randomly generated group ID and
    /// the provided parameters converted to bytes. This group ID value is what's used when publishing
    /// events to Nostr relays for the group.
    pub fn new<T1, T2, IA, IR>(
        name: T1,
        description: T2,
        admins: IA,
        relays: IR,
        image_hash: Option<[u8; 32]>,
        image_key: Option<[u8; 32]>,
        image_nonce: Option<[u8; 12]>,
    ) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
        IA: IntoIterator<Item = PublicKey>,
        IR: IntoIterator<Item = RelayUrl>,
    {
        // Generate a random 32-byte group ID
        let mut rng = OsRng;
        let random_bytes: [u8; 32] = rng.gen();

        Self {
            nostr_group_id: random_bytes,
            name: name.into(),
            description: description.into(),
            admins: admins.into_iter().collect(),
            relays: relays.into_iter().collect(),
            image_hash,
            image_key,
            image_nonce,
        }
    }

    pub(crate) fn from_raw(raw: TlsNostrGroupDataExtension) -> Result<Self, Error> {
        let mut admins = BTreeSet::new();
        for admin in raw.admin_pubkeys {
            let bytes = hex::decode(&admin)?;
            let pk = PublicKey::from_slice(&bytes)?;
            admins.insert(pk);
        }

        let mut relays = BTreeSet::new();
        for relay in raw.relays {
            let url: &str = str::from_utf8(&relay)?;
            let url = RelayUrl::parse(url)?;
            relays.insert(url);
        }

        let image_hash = if raw.image_hash.is_empty() {
            None
        } else {
            Some(
                raw.image_hash
                    .try_into()
                    .map_err(|_| Error::InvalidImageHashLength)?,
            )
        };

        let image_key = if raw.image_key.is_empty() {
            None
        } else {
            Some(
                raw.image_key
                    .try_into()
                    .map_err(|_| Error::InvalidImageKeyLength)?,
            )
        };

        let image_nonce = if raw.image_nonce.is_empty() {
            None
        } else {
            Some(
                raw.image_nonce
                    .try_into()
                    .map_err(|_| Error::InvalidImageNonceLength)?,
            )
        };

        Ok(Self {
            nostr_group_id: raw.nostr_group_id,
            name: String::from_utf8(raw.name)?,
            description: String::from_utf8(raw.description)?,
            admins,
            relays,
            image_hash,
            image_key,
            image_nonce,
        })
    }

    /// Attempts to extract and deserialize a NostrGroupDataExtension from a GroupContext.
    ///
    /// # Arguments
    ///
    /// * `group_context` - Reference to the GroupContext containing the extension
    ///
    /// # Returns
    ///
    /// * `Ok(NostrGroupDataExtension)` - Successfully extracted and deserialized extension
    /// * `Err(Error)` - Failed to find or deserialize the extension
    pub fn from_group_context(group_context: &GroupContext) -> Result<Self, Error> {
        let group_data_extension = match group_context.extensions().iter().find(|ext| {
            ext.extension_type() == ExtensionType::Unknown(NOSTR_GROUP_DATA_EXTENSION_TYPE)
        }) {
            Some(Extension::Unknown(_, ext)) => ext,
            Some(_) => return Err(Error::UnexpectedExtensionType),
            None => return Err(Error::NostrGroupDataExtensionNotFound),
        };

        let (deserialized, _) =
            TlsNostrGroupDataExtension::tls_deserialize_bytes(&group_data_extension.0)?;

        Self::from_raw(deserialized)
    }

    /// Attempts to extract and deserialize a NostrGroupDataExtension from an MlsGroup.
    ///
    /// # Arguments
    ///
    /// * `group` - Reference to the MlsGroup containing the extension
    pub fn from_group(group: &MlsGroup) -> Result<Self, Error> {
        let group_data_extension = match group.extensions().iter().find(|ext| {
            ext.extension_type() == ExtensionType::Unknown(NOSTR_GROUP_DATA_EXTENSION_TYPE)
        }) {
            Some(Extension::Unknown(_, ext)) => ext,
            Some(_) => return Err(Error::UnexpectedExtensionType),
            None => return Err(Error::NostrGroupDataExtensionNotFound),
        };

        let (deserialized, _) =
            TlsNostrGroupDataExtension::tls_deserialize_bytes(&group_data_extension.0)?;

        Self::from_raw(deserialized)
    }

    /// Returns the group ID as a hex-encoded string.
    pub fn nostr_group_id(&self) -> String {
        hex::encode(self.nostr_group_id)
    }

    /// Get nostr group data extension type
    #[inline]
    pub fn extension_type(&self) -> u16 {
        Self::EXTENSION_TYPE
    }

    /// Sets the group ID using a 32-byte array.
    ///
    /// # Arguments
    ///
    /// * `nostr_group_id` - The new 32-byte group ID
    pub fn set_nostr_group_id(&mut self, nostr_group_id: [u8; 32]) {
        self.nostr_group_id = nostr_group_id;
    }

    /// Returns the group name as a UTF-8 string.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Sets the group name.
    ///
    /// # Arguments
    ///
    /// * `name` - The new group name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Returns the group description as a UTF-8 string.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Sets the group description.
    ///
    /// # Arguments
    ///
    /// * `description` - The new group description
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Adds a new admin identity to the list.
    pub fn add_admin(&mut self, public_key: PublicKey) {
        self.admins.insert(public_key);
    }

    /// Removes an admin identity from the list if it exists.
    pub fn remove_admin(&mut self, public_key: &PublicKey) {
        self.admins.remove(public_key);
    }

    /// Adds a new relay URL to the list.
    pub fn add_relay(&mut self, relay: RelayUrl) {
        self.relays.insert(relay);
    }

    /// Removes a relay URL from the list if it exists.
    pub fn remove_relay(&mut self, relay: &RelayUrl) {
        self.relays.remove(relay);
    }

    /// Returns the group image URL.
    pub fn image_hash(&self) -> Option<&[u8; 32]> {
        self.image_hash.as_ref()
    }

    /// Sets the group image URL.
    ///
    /// # Arguments
    ///
    /// * `image` - The new image URL (optional)
    pub fn set_image_hash(&mut self, image_hash: Option<[u8; 32]>) {
        self.image_hash = image_hash;
    }

    /// Returns the group image key.
    pub fn image_key(&self) -> Option<&[u8; 32]> {
        self.image_key.as_ref()
    }

    /// Returns the group image nonce
    pub fn image_nonce(&self) -> Option<&[u8; 12]> {
        self.image_nonce.as_ref()
    }

    /// Sets the group image key.
    ///
    /// # Arguments
    ///
    /// * `image_key` - The new image encryption key (optional)
    pub fn set_image_key(&mut self, image_key: Option<[u8; 32]>) {
        self.image_key = image_key;
    }

    /// Sets the group image nonce.
    ///
    /// # Arguments
    ///
    /// * `image_nonce` - The new image encryption key (optional)
    pub fn set_image_nonce(&mut self, image_nonce: Option<[u8; 12]>) {
        self.image_nonce = image_nonce;
    }

    pub(crate) fn as_raw(&self) -> TlsNostrGroupDataExtension {
        TlsNostrGroupDataExtension {
            nostr_group_id: self.nostr_group_id,
            name: self.name.as_bytes().to_vec(),
            description: self.description.as_bytes().to_vec(),
            admin_pubkeys: self
                .admins
                .iter()
                .map(|pk| pk.to_hex().into_bytes())
                .collect(),
            relays: self
                .relays
                .iter()
                .map(|url| url.to_string().into_bytes())
                .collect(),
            image_hash: self.image_hash.map_or_else(Vec::new, |hash| hash.to_vec()),
            image_key: self.image_key.map_or_else(Vec::new, |key| key.to_vec()),
            image_nonce: self
                .image_nonce
                .map_or_else(Vec::new, |nonce| nonce.to_vec()),
        }
    }
}

#[cfg(test)]
mod tests {
    use nostr_mls_storage::test_utils::crypto_utils::generate_random_bytes;

    use super::*;

    const ADMIN_1: &str = "npub1a6awmmklxfmspwdv52qq58sk5c07kghwc4v2eaudjx2ju079cdqs2452ys";
    const ADMIN_2: &str = "npub1t5sdrgt7md8a8lf77ka02deta4vj35p3ktfskd5yz68pzmt9334qy6qks0";
    const RELAY_1: &str = "wss://relay1.com";
    const RELAY_2: &str = "wss://relay2.com";

    fn create_test_extension() -> NostrGroupDataExtension {
        let pk1 = PublicKey::parse(ADMIN_1).unwrap();
        let pk2 = PublicKey::parse(ADMIN_2).unwrap();

        let relay1 = RelayUrl::parse(RELAY_1).unwrap();
        let relay2 = RelayUrl::parse(RELAY_2).unwrap();

        let image_hash = generate_random_bytes(32).try_into().unwrap();
        let image_key = generate_random_bytes(32).try_into().unwrap();
        let image_nonce = generate_random_bytes(12).try_into().unwrap();

        NostrGroupDataExtension::new(
            "Test Group",
            "Test Description",
            [pk1, pk2],
            [relay1, relay2],
            Some(image_hash),
            Some(image_key),
            Some(image_nonce),
        )
    }

    #[test]
    fn test_new_and_getters() {
        let extension = create_test_extension();

        let pk1 = PublicKey::parse(ADMIN_1).unwrap();
        let pk2 = PublicKey::parse(ADMIN_2).unwrap();

        let relay1 = RelayUrl::parse(RELAY_1).unwrap();
        let relay2 = RelayUrl::parse(RELAY_2).unwrap();

        // Test that group_id is 32 bytes
        assert_eq!(extension.nostr_group_id.len(), 32);

        // Test basic getters
        assert_eq!(extension.name(), "Test Group");
        assert_eq!(extension.description(), "Test Description");

        assert!(extension.admins.contains(&pk1));
        assert!(extension.admins.contains(&pk2));

        assert!(extension.relays.contains(&relay1));
        assert!(extension.relays.contains(&relay2));
    }

    #[test]
    fn test_group_id_operations() {
        let mut extension = create_test_extension();
        let new_id = [42u8; 32];

        extension.set_nostr_group_id(new_id);
        assert_eq!(extension.nostr_group_id(), hex::encode(new_id));
    }

    #[test]
    fn test_name_operations() {
        let mut extension = create_test_extension();

        extension.set_name("New Name".to_string());
        assert_eq!(extension.name(), "New Name");
    }

    #[test]
    fn test_description_operations() {
        let mut extension = create_test_extension();

        extension.set_description("New Description".to_string());
        assert_eq!(extension.description(), "New Description");
    }

    #[test]
    fn test_admin_pubkey_operations() {
        let mut extension = create_test_extension();

        let admin1 = PublicKey::parse(ADMIN_1).unwrap();
        let admin2 = PublicKey::parse(ADMIN_2).unwrap();
        let admin3 =
            PublicKey::parse("npub13933f9shzt90uccjaf4p4f4arxlfcy3q6037xnx8a2kxaafrn5yqtzehs6")
                .unwrap();

        // Test add
        extension.add_admin(admin3);
        assert_eq!(extension.admins.len(), 3);
        assert!(extension.admins.contains(&admin1));
        assert!(extension.admins.contains(&admin2));
        assert!(extension.admins.contains(&admin3));

        // Test remove
        extension.remove_admin(&admin2);
        assert_eq!(extension.admins.len(), 2);
        assert!(extension.admins.contains(&admin1));
        assert!(!extension.admins.contains(&admin2)); // NOT contains
        assert!(extension.admins.contains(&admin3));
    }

    #[test]
    fn test_relay_operations() {
        let mut extension = create_test_extension();

        let relay1 = RelayUrl::parse(RELAY_1).unwrap();
        let relay2 = RelayUrl::parse(RELAY_2).unwrap();
        let relay3 = RelayUrl::parse("wss://relay3.com").unwrap();

        // Test add
        extension.add_relay(relay3.clone());
        assert_eq!(extension.relays.len(), 3);
        assert!(extension.relays.contains(&relay1));
        assert!(extension.relays.contains(&relay2));
        assert!(extension.relays.contains(&relay3));

        // Test remove
        extension.remove_relay(&relay2);
        assert_eq!(extension.relays.len(), 2);
        assert!(extension.relays.contains(&relay1));
        assert!(!extension.relays.contains(&relay2)); // NOT contains
        assert!(extension.relays.contains(&relay3));
    }

    #[test]
    fn test_image_operations() {
        let mut extension = create_test_extension();

        // Test setting image URL
        let image_hash = Some(generate_random_bytes(32).try_into().unwrap());
        extension.set_image_hash(image_hash);
        assert_eq!(extension.image_hash(), image_hash.as_ref());

        // Test setting image key
        let image_key = generate_random_bytes(32).try_into().unwrap();
        extension.set_image_key(Some(image_key));
        assert!(extension.image_key().is_some());

        // Test setting image nonce
        let image_nonce = generate_random_bytes(12).try_into().unwrap();
        extension.set_image_nonce(Some(image_nonce));
        assert!(extension.image_nonce().is_some());

        // Test clearing image
        extension.set_image_hash(None);
        extension.set_image_key(None);
        extension.set_image_nonce(None);
        assert!(extension.image_hash().is_none());
        assert!(extension.image_key().is_none());
        assert!(extension.image_nonce().is_none());
    }

    #[test]
    fn test_new_fields_in_serialization() {
        let mut extension = create_test_extension();

        // Set some image data
        let image_hash = generate_random_bytes(32).try_into().unwrap();
        let image_key = generate_random_bytes(32).try_into().unwrap();
        let image_nonce = generate_random_bytes(12).try_into().unwrap();

        extension.set_image_hash(Some(image_hash));
        extension.set_image_key(Some(image_key));
        extension.set_image_nonce(Some(image_nonce));

        // Convert to raw and back
        let raw = extension.as_raw();
        let reconstructed = NostrGroupDataExtension::from_raw(raw).unwrap();

        assert_eq!(reconstructed.image_hash(), Some(&image_hash));
        assert_eq!(reconstructed.image_nonce(), Some(&image_nonce));
        assert!(reconstructed.image_key().is_some());
        // We can't directly compare SecretKeys due to how they're implemented,
        // but we can verify the bytes are the same
        assert_eq!(reconstructed.image_key().unwrap(), &image_key);
    }

    #[test]
    fn test_serialization_overhead() {
        use tls_codec::Size;

        // Test with fixed-size vs variable-size fields
        let test_hash = [1u8; 32];
        let test_key = [2u8; 32];
        let test_nonce = [3u8; 12];

        // Create extension with Some values
        let extension_with_data = NostrGroupDataExtension::new(
            "Test",
            "Description",
            [PublicKey::parse(ADMIN_1).unwrap()],
            [RelayUrl::parse(RELAY_1).unwrap()],
            Some(test_hash),
            Some(test_key),
            Some(test_nonce),
        );

        // Create extension with None values
        let extension_without_data = NostrGroupDataExtension::new(
            "Test",
            "Description",
            [PublicKey::parse(ADMIN_1).unwrap()],
            [RelayUrl::parse(RELAY_1).unwrap()],
            None,
            None,
            None,
        );

        // Serialize both to measure size
        let with_data_raw = extension_with_data.as_raw();
        let without_data_raw = extension_without_data.as_raw();

        let with_data_size = with_data_raw.tls_serialized_len();
        let without_data_size = without_data_raw.tls_serialized_len();

        println!("With data: {} bytes", with_data_size);
        println!("Without data: {} bytes", without_data_size);
        println!(
            "Overhead difference: {} bytes",
            with_data_size as i32 - without_data_size as i32
        );

        // Check the field sizes
        println!(
            "Hash field with data: {} bytes",
            with_data_raw.image_hash.len()
        );
        println!(
            "Hash field without data: {} bytes",
            without_data_raw.image_hash.len()
        );
        println!(
            "Key field with data: {} bytes",
            with_data_raw.image_key.len()
        );
        println!(
            "Key field without data: {} bytes",
            without_data_raw.image_key.len()
        );
        println!(
            "Nonce field with data: {} bytes",
            with_data_raw.image_nonce.len()
        );
        println!(
            "Nonce field without data: {} bytes",
            without_data_raw.image_nonce.len()
        );

        // Test round-trip to ensure correctness
        let roundtrip_with = NostrGroupDataExtension::from_raw(with_data_raw).unwrap();
        let roundtrip_without = NostrGroupDataExtension::from_raw(without_data_raw).unwrap();

        // Verify data preservation
        assert_eq!(roundtrip_with.image_hash, Some(test_hash));
        assert_eq!(roundtrip_with.image_key, Some(test_key));
        assert_eq!(roundtrip_with.image_nonce, Some(test_nonce));

        assert_eq!(roundtrip_without.image_hash, None);
        assert_eq!(roundtrip_without.image_key, None);
        assert_eq!(roundtrip_without.image_nonce, None);
    }
}
