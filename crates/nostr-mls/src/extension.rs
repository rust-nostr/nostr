// Copyright (c) 2024-2025 Jeff Gardner
// Copyright (c) 2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Group Extension functionality for MLS Group Context.
//! This is a required extension for Nostr Groups as per NIP-104.

use std::collections::BTreeSet;
use std::str;

use nostr::secp256k1::rand::rngs::OsRng;
use nostr::secp256k1::rand::Rng;
use nostr::util::hex;
use nostr::{PublicKey, RelayUrl};
use openmls::extensions::{Extension, ExtensionType};
use openmls::group::{GroupContext, MlsGroup};
use tls_codec::{
    DeserializeBytes, TlsDeserialize, TlsDeserializeBytes, TlsSerialize, TlsSerializeBytes, TlsSize,
};

use crate::constant::NOSTR_GROUP_DATA_EXTENSION_TYPE;
use crate::error::Error;

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
pub(crate) struct RawNostrGroupDataExtension {
    pub nostr_group_id: [u8; 32],
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub admin_pubkeys: Vec<Vec<u8>>,
    pub relays: Vec<Vec<u8>>,
}

/// This is an MLS Group Context extension used to store the group's name,
/// description, ID, and admin identities.
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
    pub fn new<T1, T2, IA, IR>(name: T1, description: T2, admins: IA, relays: IR) -> Self
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
        }
    }

    fn from_raw(raw: RawNostrGroupDataExtension) -> Result<Self, Error> {
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

        Ok(Self {
            nostr_group_id: raw.nostr_group_id,
            name: String::from_utf8(raw.name)?,
            description: String::from_utf8(raw.description)?,
            admins,
            relays,
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
            RawNostrGroupDataExtension::tls_deserialize_bytes(&group_data_extension.0)?;

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
            RawNostrGroupDataExtension::tls_deserialize_bytes(&group_data_extension.0)?;

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

    pub(crate) fn as_raw(&self) -> RawNostrGroupDataExtension {
        RawNostrGroupDataExtension {
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
        }
    }
}

#[cfg(test)]
mod tests {
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

        NostrGroupDataExtension::new(
            "Test Group",
            "Test Description",
            [pk1, pk2],
            [relay1, relay2],
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

    // TODO: from_group_context and from_group methods would need more complex setup
    // with mocked MlsGroup and GroupContext objects to test properly
}
