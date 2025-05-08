//! Nostr MLS Group Management
//!
//! This module provides functionality for managing MLS groups in Nostr:
//! - Group creation and configuration
//! - Member management (adding/removing members)
//! - Group state updates and synchronization
//! - Group metadata handling
//! - Group secret management
//!
//! Groups in Nostr MLS have both an MLS group ID and a Nostr group ID. The MLS group ID
//! is used internally by the MLS protocol, while the Nostr group ID is used for
//! relay-based message routing and group discovery.

use std::collections::BTreeSet;
use std::str;

use nostr::{PublicKey, RelayUrl};
use nostr_mls_storage::groups::types as group_types;
use nostr_mls_storage::NostrMlsStorageProvider;
use openmls::group::GroupId;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use tls_codec::Serialize as TlsSerialize;

use super::extension::NostrGroupDataExtension;
use super::NostrMls;
use crate::error::Error;

/// Result of creating a new MLS group
#[derive(Debug)]
pub struct CreateGroupResult {
    /// The stored group
    pub group: group_types::Group,
    /// Serialized welcome message for initial group members
    pub serialized_welcome_message: Vec<u8>,
}

/// Result of updating a member's own leaf node in an MLS group
#[derive(Debug)]
pub struct SelfUpdateResult {
    /// Serialized update message to be sent to the group
    pub serialized_message: Vec<u8>,
    /// The group's exporter secret before the update
    pub current_secret: group_types::GroupExporterSecret,
    /// The group's new exporter secret after the update
    pub new_secret: group_types::GroupExporterSecret,
}

impl<Storage> NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    /// Retrieves the leaf node for the current member in an MLS group
    ///
    /// # Arguments
    ///
    /// * `group` - Reference to the MLS group
    ///
    /// # Returns
    ///
    /// * `Ok(&LeafNode)` - The leaf node for the current member
    /// * `Err(Error::OwnLeafNotFound)` - If the member's leaf node is not found
    #[inline]
    pub(crate) fn get_own_leaf<'a>(&self, group: &'a MlsGroup) -> Result<&'a LeafNode, Error> {
        group.own_leaf().ok_or(Error::OwnLeafNotFound)
    }

    /// Loads the signature key pair for the current member in an MLS group
    ///
    /// # Arguments
    ///
    /// * `group` - Reference to the MLS group
    ///
    /// # Returns
    ///
    /// * `Ok(SignatureKeyPair)` - The member's signature key pair
    /// * `Err(Error)` - If the key pair cannot be loaded
    pub(crate) fn load_mls_signer(&self, group: &MlsGroup) -> Result<SignatureKeyPair, Error> {
        let own_leaf: &LeafNode = self.get_own_leaf(group)?;
        let public_key: &[u8] = own_leaf.signature_key().as_slice();

        SignatureKeyPair::read(
            self.provider.storage(),
            public_key,
            group.ciphersuite().signature_algorithm(),
        )
        .ok_or(Error::CantLoadSigner)
    }

    /// Loads an MLS group from storage by its ID
    ///
    /// # Arguments
    ///
    /// * `mls_group_id` - The MLS group ID to load
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MlsGroup))` - The loaded group if found
    /// * `Ok(None)` - If no group exists with the given ID
    /// * `Err(Error)` - If there is an error loading the group
    pub(crate) fn load_mls_group(&self, mls_group_id: &GroupId) -> Result<Option<MlsGroup>, Error> {
        MlsGroup::load(self.provider.storage(), mls_group_id)
            .map_err(|e| Error::Provider(e.to_string()))
    }

    /// Exports the current epoch's secret key from an MLS group
    ///
    /// This secret is used for NIP-44 message encryption in Group Message Events (kind:445).
    /// The secret is cached in storage to avoid re-exporting it for each message.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    ///
    /// # Returns
    ///
    /// * `Ok(GroupExporterSecret)` - The exported secret
    /// * `Err(Error)` - If the group is not found or there is an error exporting the secret
    pub fn exporter_secret(
        &self,
        group_id: &GroupId,
    ) -> Result<group_types::GroupExporterSecret, Error> {
        let group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        match self
            .storage()
            .get_group_exporter_secret(group_id, group.epoch().as_u64())
            .map_err(|e| Error::Group(e.to_string()))?
        {
            Some(group_exporter_secret) => Ok(group_exporter_secret),
            // If it's not already in the storage, export the secret and save it
            None => {
                let export_secret: [u8; 32] = group
                    .export_secret(&self.provider, "nostr", b"nostr", 32)?
                    .try_into()
                    .map_err(|_| {
                        Error::Group("Failed to convert export secret to [u8; 32]".to_string())
                    })?;
                let group_exporter_secret = group_types::GroupExporterSecret {
                    mls_group_id: group_id.clone(),
                    epoch: group.epoch().as_u64(),
                    secret: export_secret,
                };

                self.storage()
                    .save_group_exporter_secret(group_exporter_secret.clone())
                    .map_err(|e| Error::Group(e.to_string()))?;

                Ok(group_exporter_secret)
            }
        }
    }

    /// Retrieves a Nostr MLS group by its MLS group ID
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID to look up
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Group))` - The group if found
    /// * `Ok(None)` - If no group exists with the given ID
    /// * `Err(Error)` - If there is an error accessing storage
    pub fn get_group(&self, group_id: &GroupId) -> Result<Option<group_types::Group>, Error> {
        self.storage()
            .find_group_by_mls_group_id(group_id)
            .map_err(|e| Error::Group(e.to_string()))
    }

    /// Retrieves all Nostr MLS groups from storage
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Group>)` - List of all groups
    /// * `Err(Error)` - If there is an error accessing storage
    pub fn get_groups(&self) -> Result<Vec<group_types::Group>, Error> {
        self.storage()
            .all_groups()
            .map_err(|e| Error::Group(e.to_string()))
    }

    /// Gets the public keys of all members in an MLS group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    ///
    /// # Returns
    ///
    /// * `Ok(BTreeSet<PublicKey>)` - Set of member public keys
    /// * `Err(Error)` - If the group is not found or there is an error accessing member data
    pub fn get_members(&self, group_id: &GroupId) -> Result<BTreeSet<PublicKey>, Error> {
        let group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        // Store members in a variable to extend its lifetime
        let mut members = group.members();
        members.try_fold(BTreeSet::new(), |mut acc, m| {
            let credentials: BasicCredential = BasicCredential::try_from(m.credential)?;
            let hex_bytes: &[u8] = credentials.identity();
            let hex_str: &str = str::from_utf8(hex_bytes)?;
            let public_key = PublicKey::from_hex(hex_str)?;
            acc.insert(public_key);
            Ok(acc)
        })
    }

    /// Retrieves the set of relay URLs associated with an MLS group
    ///
    /// # Arguments
    ///
    /// * `mls_group_id` - The MLS group ID
    ///
    /// # Returns
    ///
    /// * `Ok(BTreeSet<RelayUrl>)` - Set of relay URLs where group messages are published
    /// * `Err(Error)` - If there is an error accessing storage or the group is not found
    pub fn get_relays(&self, mls_group_id: &GroupId) -> Result<BTreeSet<RelayUrl>, Error> {
        let relays = self
            .storage()
            .group_relays(mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?;
        Ok(relays.into_iter().map(|r| r.relay_url).collect())
    }

    /// Creates a new MLS group with the specified members and settings.
    ///
    /// This function creates a new MLS group with the given name, description, members, and administrators.
    /// It generates the necessary cryptographic credentials, configures the group with Nostr-specific extensions,
    /// and adds the specified members.
    ///
    /// # Arguments
    ///
    /// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
    /// * `name` - The name of the group
    /// * `description` - A description of the group
    /// * `member_key_packages` - A vector of KeyPackages for the initial group members
    /// * `admin_pubkeys_hex` - A vector of hex-encoded Nostr public keys for group administrators
    /// * `creator_pubkey_hex` - The hex-encoded Nostr public key of the group creator
    /// * `group_relays` - A vector of relay URLs where group messages will be published
    ///
    /// # Returns
    ///
    /// A `CreateGroupResult` containing:
    /// - The created MLS group
    /// - A serialized welcome message for the initial members
    /// - The Nostr-specific group data
    ///
    /// # Errors
    ///
    /// Returns a `Error` if:
    /// - Credential generation fails
    /// - Group creation fails
    /// - Adding members fails
    /// - Message serialization fails
    pub fn create_group<S1, S2>(
        &self,
        name: S1,
        description: S2,
        creator_public_key: &PublicKey,
        member_pubkeys: &[PublicKey],
        member_key_packages: &[KeyPackage],
        admins: Vec<PublicKey>,
        group_relays: Vec<RelayUrl>,
    ) -> Result<CreateGroupResult, Error>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        // Validate group members
        self.validate_group_members(creator_public_key, member_pubkeys, &admins)?;

        let (credential, signer) = self.generate_credential_with_key(creator_public_key)?;

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Credential and signer created, {:?}",
            credential
        );

        let group_data =
            NostrGroupDataExtension::new(name, description, admins, group_relays.clone());

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group data created, {:?}",
            group_data
        );

        let serialized_group_data = group_data
            .as_raw()
            .tls_serialize_detached()
            .expect("Failed to serialize group data");

        let extensions = vec![Extension::Unknown(
            group_data.extension_type(),
            UnknownExtension(serialized_group_data),
        )];
        let extensions =
            Extensions::from_vec(extensions).expect("Couldn't convert extensions vec to Object");

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group config extensions created, {:?}",
            extensions
        );

        // Build the group config
        let capabilities = self.capabilities();
        let group_config = MlsGroupCreateConfig::builder()
            .ciphersuite(self.ciphersuite)
            .use_ratchet_tree_extension(true)
            .capabilities(capabilities)
            .with_group_context_extensions(extensions)?
            .build();

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group config built, {:?}",
            group_config
        );

        let mut mls_group =
            MlsGroup::new(&self.provider, &signer, &group_config, credential.clone())?;

        // Add members to the group
        let (_, welcome_out, _group_info) =
            mls_group.add_members(&self.provider, &signer, member_key_packages)?;

        // Merge the pending commit adding the memebers
        mls_group.merge_pending_commit(&self.provider)?;

        // Serialize the welcome message and send it to the members
        let serialized_welcome_message = welcome_out.tls_serialize_detached()?;

        let group_type = if mls_group.members().count() > 2 {
            group_types::GroupType::Group
        } else {
            group_types::GroupType::DirectMessage
        };

        // Save the NostrMLS Group
        let group = group_types::Group {
            mls_group_id: mls_group.group_id().clone(),
            nostr_group_id: group_data.clone().nostr_group_id,
            name: group_data.clone().name,
            description: group_data.clone().description,
            admin_pubkeys: group_data.clone().admins,
            last_message_id: None,
            last_message_at: None,
            group_type,
            epoch: mls_group.epoch().as_u64(),
            state: group_types::GroupState::Active,
        };

        self.storage().save_group(group.clone()).map_err(
            |e: nostr_mls_storage::groups::error::GroupError| Error::Group(e.to_string()),
        )?;

        // Always (re-)save the group relays after saving the group
        for relay_url in group_relays.into_iter() {
            let group_relay = group_types::GroupRelay {
                mls_group_id: group.mls_group_id.clone(),
                relay_url,
            };

            self.storage()
                .save_group_relay(group_relay)
                .map_err(|e| Error::Group(e.to_string()))?;
        }

        Ok(CreateGroupResult {
            group,
            serialized_welcome_message,
        })
    }

    /// Updates the current member's leaf node in an MLS group.
    /// Does not currently support updating any group attributes.
    ///
    /// This function performs a self-update operation in the specified MLS group by:
    /// 1. Loading the group from storage
    /// 2. Generating a new signature keypair
    /// 3. Storing the keypair
    /// 4. Creating and applying a self-update proposal
    ///
    /// # Arguments
    ///
    /// * `nostr_mls` - The NostrMls instance containing MLS configuration and provider
    /// * `mls_group_id` - The ID of the MLS group as a byte vector
    ///
    /// # Returns
    ///
    /// A Result containing a tuple of:
    /// - MlsMessageOut: The self-update message to be sent to the group
    /// - Option<MlsMessageOut>: Optional welcome message if new members are added
    /// - Option<GroupInfo>: Optional updated group info
    ///
    /// # Errors
    ///
    /// Returns a Error if:
    /// - The group cannot be loaded from storage
    /// - The specified group is not found
    /// - Failed to generate or store signature keypair
    /// - Failed to perform self-update operation
    pub fn self_update(&self, group_id: &GroupId) -> Result<SelfUpdateResult, Error> {
        // Load group
        let mut group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let current_secret: group_types::GroupExporterSecret = self
            .storage()
            .get_group_exporter_secret(group_id, group.epoch().as_u64())
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupExporterSecretNotFound)?;

        tracing::debug!(target: "nostr_openmls::groups::self_update", "Current epoch: {:?}", current_secret.epoch);

        // Load current signer
        let current_signer: SignatureKeyPair = self.load_mls_signer(&group)?;

        // Get own leaf
        let own_leaf = self.get_own_leaf(&group)?;

        let new_signature_keypair = SignatureKeyPair::new(self.ciphersuite.signature_algorithm())?;

        new_signature_keypair
            .store(self.provider.storage())
            .map_err(|e| Error::Provider(e.to_string()))?;

        let pubkey = BasicCredential::try_from(own_leaf.credential().clone())?
            .identity()
            .to_vec();

        let new_credential: BasicCredential = BasicCredential::new(pubkey);
        let new_credential_with_key = CredentialWithKey {
            credential: new_credential.into(),
            signature_key: new_signature_keypair.public().into(),
        };

        let leaf_node_params = LeafNodeParameters::builder()
            .with_credential_with_key(new_credential_with_key)
            .with_capabilities(own_leaf.capabilities().clone())
            .with_extensions(own_leaf.extensions().clone())
            .build();

        let commit_message_bundle = group.self_update_with_new_signer(
            &self.provider,
            &current_signer,
            &new_signature_keypair,
            leaf_node_params,
        )?;

        // Merge the commit
        group.merge_pending_commit(&self.provider)?;

        // Export the new epoch's exporter secret
        let new_secret = self.exporter_secret(group_id)?;

        tracing::debug!(target: "nostr_openmls::groups::self_update", "New epoch: {:?}", new_secret.epoch);

        // Serialize the message
        let serialized_message = commit_message_bundle.commit().tls_serialize_detached()?;

        Ok(SelfUpdateResult {
            serialized_message,
            current_secret,
            new_secret,
        })
    }

    /// Validates the members and admins of a group during creation
    ///
    /// # Arguments
    /// * `creator_pubkey` - The public key of the group creator
    /// * `member_pubkeys` - List of public keys for group members
    /// * `admin_pubkeys` - List of public keys for group admins
    ///
    /// # Returns
    /// * `Ok(true)` if validation passes
    /// * `Err(GroupError::InvalidParameters)` if validation fails
    ///
    /// # Validation Rules
    /// - Creator must be an admin but not included in member list
    /// - All admins must also be members (except creator)
    ///
    /// # Errors
    /// Returns `GroupError::InvalidParameters` with descriptive message if:
    /// - Creator is not an admin
    /// - Creator is in member list
    /// - Any admin, other than the creator, is not a member
    fn validate_group_members(
        &self,
        creator_pubkey: &PublicKey,
        member_pubkeys: &[PublicKey],
        admin_pubkeys: &[PublicKey],
    ) -> Result<bool, Error> {
        // Creator must be an admin
        if !admin_pubkeys.contains(creator_pubkey) {
            return Err(Error::Group("Creator must be an admin".to_string()));
        }

        // Creator must not be included as a member
        if member_pubkeys.contains(creator_pubkey) {
            return Err(Error::Group(
                "Creator must not be included as a member".to_string(),
            ));
        }

        // Check that admins are valid pubkeys and are members
        for pubkey in admin_pubkeys.iter() {
            if !member_pubkeys.contains(pubkey) && creator_pubkey != pubkey {
                return Err(Error::Group("Admin must be a member".to_string()));
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use nostr::{Keys, PublicKey};

    use crate::tests::create_test_nostr_mls;

    fn create_test_group_members() -> (PublicKey, Vec<PublicKey>, Vec<PublicKey>) {
        let creator = Keys::generate();
        let member1 = Keys::generate();
        let member2 = Keys::generate();

        let creator_pk = creator.public_key();
        let members = vec![member1.public_key(), member2.public_key()];
        let admins = vec![creator_pk, member1.public_key()];

        (creator_pk, members, admins)
    }

    #[test]
    fn test_validate_group_members() {
        let nostr_mls = create_test_nostr_mls();
        let (creator_pk, members, admins) = create_test_group_members();

        // Test valid configuration
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &members, &admins)
            .is_ok());

        // Test creator not in admin list
        let bad_admins = vec![members[0]];
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &members, &bad_admins)
            .is_err());

        // Test creator in member list
        let bad_members = vec![creator_pk, members[0]];
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &bad_members, &admins)
            .is_err());

        // Test admin not in member list
        let non_member = Keys::generate().public_key();
        let bad_admins = vec![creator_pk, non_member];
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &members, &bad_admins)
            .is_err());
    }
}
