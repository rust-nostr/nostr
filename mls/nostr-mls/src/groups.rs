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

/// Result of updating a group
#[derive(Debug)]
pub struct UpdateGroupResult {
    /// Serialized commit message to be fanned out to the group
    pub serialized_commit_message: Vec<u8>,
    /// Optional serialized welcome message for new members
    pub serialized_welcome_message: Option<Vec<u8>>,
    /// Optional serialized group info for the group - since we use the ratchet_tree extension this should always be present
    pub serialized_group_info: Option<Vec<u8>>,
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

    /// Gets the current user's public key from an MLS group
    ///
    /// # Arguments
    ///
    /// * `group` - Reference to the MLS group
    ///
    /// # Returns
    ///
    /// * `Ok(PublicKey)` - The current user's public key
    /// * `Err(Error)` - If the user's leaf node is not found or there is an error extracting the public key
    pub(crate) fn get_own_pubkey(&self, group: &MlsGroup) -> Result<PublicKey, Error> {
        let own_leaf = self.get_own_leaf(group)?;
        let credentials: BasicCredential =
            BasicCredential::try_from(own_leaf.credential().clone())?;
        let hex_bytes: &[u8] = credentials.identity();
        let hex_str: &str = str::from_utf8(hex_bytes)?;
        let public_key = PublicKey::from_hex(hex_str)?;
        Ok(public_key)
    }

    /// Checks if the current user (own leaf node) is an admin of an MLS group
    ///
    /// # Arguments
    ///
    /// * `group` - Reference to the MLS group
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - The current user is an admin
    /// * `Ok(false)` - The current user is not an admin
    /// * `Err(Error)` - If the current user's public key cannot be extracted or the group is not found
    pub(crate) fn is_admin(&self, group: &MlsGroup) -> Result<bool, Error> {
        let current_user_pubkey = self.get_own_pubkey(group)?;
        let stored_group = self
            .get_group(group.group_id())?
            .ok_or(Error::GroupNotFound)?;
        Ok(stored_group.admin_pubkeys.contains(&current_user_pubkey))
    }

    /// Extracts the public key from a leaf node
    ///
    /// # Arguments
    ///
    /// * `leaf` - Reference to the leaf node
    ///
    /// # Returns
    ///
    /// * `Ok(PublicKey)` - The public key extracted from the leaf node
    /// * `Err(Error)` - If the public key cannot be extracted or there is an error converting the public key to hex
    pub(crate) fn pubkey_for_member(&self, member: &Member) -> Result<PublicKey, Error> {
        let credentials: BasicCredential = BasicCredential::try_from(member.credential.clone())?;
        let hex_bytes: &[u8] = credentials.identity();
        let hex_str: &str = str::from_utf8(hex_bytes)?;
        let public_key = PublicKey::from_hex(hex_str)?;
        Ok(public_key)
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

    /// Add members to a group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `key_packages` - The key packages for the new members
    ///
    /// # Returns
    ///
    /// * `Ok(AddMembersResult)` - The result of adding members
    /// * `Err(Error)` - If there is an error adding members
    pub fn add_members(
        &self,
        group_id: &GroupId,
        key_packages: &[KeyPackage],
    ) -> Result<UpdateGroupResult, Error> {
        // Load group
        let mut group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let signer: SignatureKeyPair = self.load_mls_signer(&group)?;

        // Check if current user is an admin
        if !self.is_admin(&group)? {
            return Err(Error::Group(
                "Only group admins can add members".to_string(),
            ));
        }

        let (commit_message, welcome_message, group_info) = group
            .add_members(&self.provider, &signer, key_packages)
            .map_err(|e| Error::Group(e.to_string()))?;

        group
            .merge_pending_commit(&self.provider)
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_commit_message = commit_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_welcome_message = welcome_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_group_info = group_info
            .map(|g| {
                g.tls_serialize_detached()
                    .map_err(|e| Error::Group(e.to_string()))
            })
            .transpose()?;

        Ok(UpdateGroupResult {
            serialized_commit_message,
            serialized_welcome_message: Some(serialized_welcome_message),
            serialized_group_info,
        })
    }

    /// Remove members from a group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `pubkeys_hex` - The hex-encoded Nostr public keys of the members to remove
    ///
    /// # Returns
    ///
    /// * `Ok(UpdateGroupResult)` - The result of removing members
    /// * `Err(Error)` - If there is an error removing members
    pub fn remove_members(
        &self,
        group_id: &GroupId,
        pubkeys: &[PublicKey],
    ) -> Result<UpdateGroupResult, Error> {
        // Load group
        let mut group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let signer: SignatureKeyPair = self.load_mls_signer(&group)?;

        // Check if current user is an admin
        if !self.is_admin(&group)? {
            return Err(Error::Group(
                "Only group admins can remove members".to_string(),
            ));
        }

        // Convert pubkeys to leaf indices
        let mut leaf_indices = Vec::new();
        let members = group.members();

        for (index, member) in members.enumerate() {
            let pubkey = self.pubkey_for_member(&member)?;
            if pubkeys.contains(&pubkey) {
                leaf_indices.push(LeafNodeIndex::new(index as u32));
            }
        }

        if leaf_indices.is_empty() {
            return Err(Error::Group(
                "No matching members found to remove".to_string(),
            ));
        }

        let (commit_message, welcome_option, group_info) = group
            .remove_members(&self.provider, &signer, &leaf_indices)
            .map_err(|e| Error::Group(e.to_string()))?;

        group
            .merge_pending_commit(&self.provider)
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_commit_message = commit_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_welcome_message = welcome_option
            .map(|w| {
                w.tls_serialize_detached()
                    .map_err(|e| Error::Group(e.to_string()))
            })
            .transpose()?;

        let serialized_group_info = group_info
            .map(|g| {
                g.tls_serialize_detached()
                    .map_err(|e| Error::Group(e.to_string()))
            })
            .transpose()?;

        Ok(UpdateGroupResult {
            serialized_commit_message,
            serialized_welcome_message,
            serialized_group_info,
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
    pub fn self_update(&self, group_id: &GroupId) -> Result<UpdateGroupResult, Error> {
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
        let serialized_commit_message = commit_message_bundle.commit().tls_serialize_detached()?;

        let serialized_welcome_message = commit_message_bundle
            .welcome()
            .map(|w| {
                w.tls_serialize_detached()
                    .map_err(|e| Error::Group(e.to_string()))
            })
            .transpose()?;

        let serialized_group_info = commit_message_bundle
            .group_info()
            .map(|g| {
                g.tls_serialize_detached()
                    .map_err(|e| Error::Group(e.to_string()))
            })
            .transpose()?;

        Ok(UpdateGroupResult {
            serialized_commit_message,
            serialized_welcome_message,
            serialized_group_info,
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
    use openmls::prelude::BasicCredential;

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

    #[test]
    fn test_add_members_success() {
        use nostr::RelayUrl;
        use openmls::prelude::KeyPackage;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            // Generate a credential and create a key package directly
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the initial group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for add_members testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Verify initial group state
        let initial_member_count = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get initial members")
            .len();
        assert_eq!(initial_member_count, 3); // creator + 2 initial members

        // Now add new members
        let new_member1 = Keys::generate();
        let new_member2 = Keys::generate();
        let new_member_pks = vec![new_member1.public_key(), new_member2.public_key()];

        // Create key packages for new members
        let mut new_key_packages = Vec::new();
        for member_pk in &new_member_pks {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            new_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Test add_members
        let add_result = creator_nostr_mls
            .add_members(group_id, &new_key_packages)
            .expect("Failed to add members");

        // Verify the result contains the expected data
        assert!(
            !add_result.serialized_commit_message.is_empty(),
            "Commit message should not be empty"
        );
        assert!(
            add_result.serialized_welcome_message.is_some(),
            "Welcome message should not be empty"
        );

        // Verify the group state was updated correctly
        let final_members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get final members");
        assert_eq!(final_members.len(), 5); // creator + 2 initial + 2 new = 5 total

        // Verify new members are in the group
        for new_member_pk in &new_member_pks {
            assert!(
                final_members.contains(new_member_pk),
                "New member should be in the group"
            );
        }

        // Verify original members are still in the group
        assert!(
            final_members.contains(&creator_pk),
            "Creator should still be in group"
        );
        for initial_member_pk in &initial_members {
            assert!(
                final_members.contains(initial_member_pk),
                "Initial member should still be in group"
            );
        }
    }

    #[test]
    fn test_add_members_group_not_found() {
        use openmls::group::GroupId;

        let nostr_mls = create_test_nostr_mls();
        let non_existent_group_id = GroupId::from_slice(&[1, 2, 3, 4, 5]);

        let result = nostr_mls.add_members(&non_existent_group_id, &[]);
        assert!(
            matches!(result, Err(crate::Error::GroupNotFound)),
            "Should return GroupNotFound error for non-existent group"
        );
    }

    #[test]
    fn test_add_members_empty_key_packages() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the initial group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for empty add_members testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Get initial member count
        let initial_member_count = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get initial members")
            .len();

        // Test adding empty key packages (should be a no-op but not error)
        let add_result = creator_nostr_mls.add_members(group_id, &[]);

        // This might error or succeed depending on MLS implementation
        // If it succeeds, verify no members were added
        if let Ok(_result) = add_result {
            let final_member_count = creator_nostr_mls
                .get_members(group_id)
                .expect("Failed to get final members")
                .len();
            assert_eq!(
                initial_member_count, final_member_count,
                "Member count should not change when adding empty key packages"
            );
        }
        // If it errors, that's also acceptable behavior
    }

    #[test]
    fn test_add_members_epoch_advancement() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the initial group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for epoch advancement testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Get initial epoch
        let initial_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get group")
            .expect("Group should exist");
        let initial_epoch = initial_group.epoch;

        // Create key package for new member
        let new_member = Keys::generate();
        let new_member_nostr_mls = create_test_nostr_mls();
        let (credential, signature_keypair) = new_member_nostr_mls
            .generate_credential_with_key(&new_member.public_key())
            .expect("Failed to generate credential");

        let capabilities = new_member_nostr_mls.capabilities();
        let key_package_bundle = openmls::prelude::KeyPackage::builder()
            .leaf_node_capabilities(capabilities)
            .mark_as_last_resort()
            .build(
                new_member_nostr_mls.ciphersuite,
                &new_member_nostr_mls.provider,
                &signature_keypair,
                credential,
            )
            .expect("Failed to build key package");

        // Add the new member
        let _add_result = creator_nostr_mls
            .add_members(group_id, &[key_package_bundle.key_package().clone()])
            .expect("Failed to add member");

        // Verify the MLS group epoch was advanced by checking the actual MLS group
        let mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let final_mls_epoch = mls_group.epoch().as_u64();

        assert!(
            final_mls_epoch > initial_epoch,
            "MLS group epoch should advance after adding members (initial: {}, final: {})",
            initial_epoch,
            final_mls_epoch
        );

        // Verify the new member was added
        let final_members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get members");
        assert!(
            final_members.contains(&new_member.public_key()),
            "New member should be in the group"
        );
        assert_eq!(
            final_members.len(),
            4, // creator + 2 initial + 1 new = 4 total
            "Should have 4 total members"
        );
    }

    #[test]
    fn test_get_own_pubkey() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for get_own_pubkey testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Load the MLS group
        let mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");

        // Test get_own_pubkey
        let own_pubkey = creator_nostr_mls
            .get_own_pubkey(&mls_group)
            .expect("Failed to get own pubkey");

        assert_eq!(
            own_pubkey, creator_pk,
            "Own pubkey should match creator pubkey"
        );
    }

    #[test]
    fn test_is_admin() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for is_admin testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Load the MLS group
        let mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");

        // Test is_admin - creator should be admin
        let is_admin = creator_nostr_mls
            .is_admin(&mls_group)
            .expect("Failed to check admin status");

        assert!(is_admin, "Creator should be admin");
    }

    #[test]
    fn test_admin_permission_checks() {
        use nostr::RelayUrl;

        let admin_nostr_mls = create_test_nostr_mls();
        let non_admin_nostr_mls = create_test_nostr_mls();

        // Generate keys
        let admin_keys = Keys::generate();
        let non_admin_keys = Keys::generate();
        let member1_keys = Keys::generate();

        let admin_pk = admin_keys.public_key();
        let non_admin_pk = non_admin_keys.public_key();
        let member1_pk = member1_keys.public_key();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();

        // Key package for non-admin user
        let (credential, signature_keypair) = admin_nostr_mls
            .generate_credential_with_key(&non_admin_pk)
            .expect("Failed to generate credential");
        let capabilities = admin_nostr_mls.capabilities();
        let key_package_bundle = openmls::prelude::KeyPackage::builder()
            .leaf_node_capabilities(capabilities)
            .mark_as_last_resort()
            .build(
                admin_nostr_mls.ciphersuite,
                &admin_nostr_mls.provider,
                &signature_keypair,
                credential,
            )
            .expect("Failed to build key package");
        initial_key_packages.push(key_package_bundle.key_package().clone());

        // Key package for member1
        let (credential, signature_keypair) = admin_nostr_mls
            .generate_credential_with_key(&member1_pk)
            .expect("Failed to generate credential");
        let capabilities2 = admin_nostr_mls.capabilities();
        let key_package_bundle = openmls::prelude::KeyPackage::builder()
            .leaf_node_capabilities(capabilities2)
            .mark_as_last_resort()
            .build(
                admin_nostr_mls.ciphersuite,
                &admin_nostr_mls.provider,
                &signature_keypair,
                credential,
            )
            .expect("Failed to build key package");
        initial_key_packages.push(key_package_bundle.key_package().clone());

        // Create group with admin as creator, non_admin and member1 as members
        // Only admin is an admin
        let create_result = admin_nostr_mls
            .create_group(
                "Admin Test Group",
                "A test group for admin permission testing",
                &admin_pk,
                &[non_admin_pk, member1_pk],
                &initial_key_packages,
                vec![admin_pk], // Only admin is an admin
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Now let's simulate the non-admin user trying to add/remove members
        // First, we need to set up the non-admin user's MLS group state
        // In a real scenario, they would have joined via the welcome message

        // Create a new member to add
        let new_member_keys = Keys::generate();
        let new_member_pk = new_member_keys.public_key();

        // Create key package for new member
        let (credential, signature_keypair) = non_admin_nostr_mls
            .generate_credential_with_key(&new_member_pk)
            .expect("Failed to generate credential");
        let capabilities3 = non_admin_nostr_mls.capabilities();
        let new_key_package_bundle = openmls::prelude::KeyPackage::builder()
            .leaf_node_capabilities(capabilities3)
            .mark_as_last_resort()
            .build(
                non_admin_nostr_mls.ciphersuite,
                &non_admin_nostr_mls.provider,
                &signature_keypair,
                credential,
            )
            .expect("Failed to build key package");

        // Test that admin can add members (should work)
        let add_result =
            admin_nostr_mls.add_members(group_id, &[new_key_package_bundle.key_package().clone()]);
        assert!(add_result.is_ok(), "Admin should be able to add members");

        // Test that admin can remove members (should work)
        let remove_result = admin_nostr_mls.remove_members(group_id, &[member1_pk]);
        assert!(
            remove_result.is_ok(),
            "Admin should be able to remove members"
        );

        // Note: Testing non-admin permissions would require the non-admin user to actually
        // be part of the MLS group, which would require processing the welcome message.
        // For now, we've verified that admin permissions work correctly.
    }

    #[test]
    fn test_pubkey_for_member() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for pubkey_for_member testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Load the MLS group
        let mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");

        // Test pubkey_for_member by checking all members
        let members: Vec<_> = mls_group.members().collect();
        let mut found_pubkeys = Vec::new();

        for member in &members {
            let pubkey = creator_nostr_mls
                .pubkey_for_member(member)
                .expect("Failed to get pubkey for member");
            found_pubkeys.push(pubkey);
        }

        // Verify we found the expected public keys
        assert!(
            found_pubkeys.contains(&creator_pk),
            "Should find creator pubkey"
        );
        for member_pk in &initial_members {
            assert!(
                found_pubkeys.contains(member_pk),
                "Should find member pubkey: {:?}",
                member_pk
            );
        }
        assert_eq!(found_pubkeys.len(), 3, "Should have 3 members total");
    }

    #[test]
    fn test_remove_members_success() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for remove_members testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Verify initial group state
        let initial_members_set = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get initial members");
        assert_eq!(initial_members_set.len(), 3); // creator + 2 initial members

        // Remove one member (the second initial member)
        let member_to_remove = initial_members[1];
        let remove_result = creator_nostr_mls
            .remove_members(group_id, &[member_to_remove])
            .expect("Failed to remove member");

        // Verify the result contains the expected data
        assert!(
            !remove_result.serialized_commit_message.is_empty(),
            "Commit message should not be empty"
        );

        // Verify the group state was updated correctly
        let final_members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get final members");
        assert_eq!(final_members.len(), 2); // creator + 1 remaining member

        // Verify the removed member is no longer in the group
        assert!(
            !final_members.contains(&member_to_remove),
            "Removed member should not be in the group"
        );

        // Verify remaining members are still in the group
        assert!(
            final_members.contains(&creator_pk),
            "Creator should still be in group"
        );
        assert!(
            final_members.contains(&initial_members[0]),
            "Remaining member should still be in group"
        );
    }

    #[test]
    fn test_remove_members_group_not_found() {
        use openmls::group::GroupId;

        let nostr_mls = create_test_nostr_mls();
        let non_existent_group_id = GroupId::from_slice(&[1, 2, 3, 4, 5]);
        let dummy_pubkey = Keys::generate().public_key();

        let result = nostr_mls.remove_members(&non_existent_group_id, &[dummy_pubkey]);
        assert!(
            matches!(result, Err(crate::Error::GroupNotFound)),
            "Should return GroupNotFound error for non-existent group"
        );
    }

    #[test]
    fn test_remove_members_no_matching_members() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for remove_members no matching testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Try to remove a member that doesn't exist in the group
        let non_member = Keys::generate().public_key();
        let result = creator_nostr_mls.remove_members(group_id, &[non_member]);

        assert!(
            matches!(
                result,
                Err(crate::Error::Group(ref msg)) if msg.contains("No matching members found")
            ),
            "Should return error when no matching members found"
        );
    }

    #[test]
    fn test_remove_members_epoch_advancement() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for remove_members epoch testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Get initial epoch
        let initial_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get group")
            .expect("Group should exist");
        let initial_epoch = initial_group.epoch;

        // Remove a member
        let member_to_remove = initial_members[0];
        let _remove_result = creator_nostr_mls
            .remove_members(group_id, &[member_to_remove])
            .expect("Failed to remove member");

        // Verify the MLS group epoch was advanced
        let mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let final_mls_epoch = mls_group.epoch().as_u64();

        assert!(
            final_mls_epoch > initial_epoch,
            "MLS group epoch should advance after removing members (initial: {}, final: {})",
            initial_epoch,
            final_mls_epoch
        );

        // Verify the member was removed
        let final_members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get members");
        assert!(
            !final_members.contains(&member_to_remove),
            "Removed member should not be in the group"
        );
        assert_eq!(
            final_members.len(),
            2, // creator + 1 remaining member
            "Should have 2 total members after removal"
        );
    }

    #[test]
    fn test_self_update_success() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for self_update testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Verify initial group state
        let initial_members_set = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get initial members");
        assert_eq!(initial_members_set.len(), 3); // creator + 2 initial members

        // Get initial group state
        let initial_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let initial_epoch = initial_mls_group.epoch().as_u64();

        // Ensure the exporter secret exists before self update (this creates it if it doesn't exist)
        let _initial_secret = creator_nostr_mls
            .exporter_secret(group_id)
            .expect("Failed to get initial exporter secret");

        // Perform self update
        let update_result = creator_nostr_mls
            .self_update(group_id)
            .expect("Failed to perform self update");

        // Verify the result contains the expected data
        assert!(
            !update_result.serialized_commit_message.is_empty(),
            "Commit message should not be empty"
        );
        // Note: self_update typically doesn't produce a welcome message unless there are special circumstances
        // assert!(update_result.serialized_welcome_message.is_none(), "Welcome message should typically be None for self-update");

        // Verify the group state was updated correctly
        let final_members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get final members");
        assert_eq!(
            final_members.len(),
            3,
            "Member count should remain the same after self update"
        );

        // Verify all original members are still in the group
        assert!(
            final_members.contains(&creator_pk),
            "Creator should still be in group"
        );
        for initial_member_pk in &initial_members {
            assert!(
                final_members.contains(initial_member_pk),
                "Initial member should still be in group"
            );
        }

        // Verify the epoch was advanced
        let final_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let final_epoch = final_mls_group.epoch().as_u64();

        assert!(
            final_epoch > initial_epoch,
            "Epoch should advance after self update (initial: {}, final: {})",
            initial_epoch,
            final_epoch
        );
    }

    #[test]
    fn test_self_update_group_not_found() {
        use openmls::group::GroupId;

        let nostr_mls = create_test_nostr_mls();
        let non_existent_group_id = GroupId::from_slice(&[1, 2, 3, 4, 5]);

        let result = nostr_mls.self_update(&non_existent_group_id);
        assert!(
            matches!(result, Err(crate::Error::GroupNotFound)),
            "Should return GroupNotFound error for non-existent group"
        );
    }

    #[test]
    fn test_self_update_key_rotation() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for self_update key rotation testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Get initial signature key from the leaf node
        let initial_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let initial_own_leaf = creator_nostr_mls
            .get_own_leaf(&initial_mls_group)
            .expect("Failed to get initial own leaf");
        let initial_signature_key = initial_own_leaf.signature_key().as_slice().to_vec();

        // Ensure the exporter secret exists before self update (this creates it if it doesn't exist)
        let _initial_secret = creator_nostr_mls
            .exporter_secret(group_id)
            .expect("Failed to get initial exporter secret");

        // Perform self update (this should rotate the signing key)
        let _update_result = creator_nostr_mls
            .self_update(group_id)
            .expect("Failed to perform self update");

        // Get the new signature key
        let final_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let final_own_leaf = creator_nostr_mls
            .get_own_leaf(&final_mls_group)
            .expect("Failed to get final own leaf");
        let final_signature_key = final_own_leaf.signature_key().as_slice().to_vec();

        // Verify the signature key has been rotated
        assert_ne!(
            initial_signature_key, final_signature_key,
            "Signature key should be different after self update"
        );

        // Verify the public key identity remains the same
        let initial_credential = BasicCredential::try_from(initial_own_leaf.credential().clone())
            .expect("Failed to extract initial credential");
        let final_credential = BasicCredential::try_from(final_own_leaf.credential().clone())
            .expect("Failed to extract final credential");

        assert_eq!(
            initial_credential.identity(),
            final_credential.identity(),
            "Public key identity should remain the same after self update"
        );
    }

    #[test]
    fn test_self_update_exporter_secret_rotation() {
        use nostr::RelayUrl;

        let creator_nostr_mls = create_test_nostr_mls();
        let (creator_pk, initial_members, admins) = create_test_group_members();

        // Create key packages for initial members
        let mut initial_key_packages = Vec::new();
        for member_pk in &initial_members {
            let member_nostr_mls = create_test_nostr_mls();
            let (credential, signature_keypair) = member_nostr_mls
                .generate_credential_with_key(member_pk)
                .expect("Failed to generate credential");

            let capabilities = member_nostr_mls.capabilities();
            let key_package_bundle = openmls::prelude::KeyPackage::builder()
                .leaf_node_capabilities(capabilities)
                .mark_as_last_resort()
                .build(
                    member_nostr_mls.ciphersuite,
                    &member_nostr_mls.provider,
                    &signature_keypair,
                    credential,
                )
                .expect("Failed to build key package");

            initial_key_packages.push(key_package_bundle.key_package().clone());
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                "Test Group",
                "A test group for self_update exporter secret testing",
                &creator_pk,
                &initial_members,
                &initial_key_packages,
                admins,
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Get initial exporter secret
        let initial_secret = creator_nostr_mls
            .exporter_secret(group_id)
            .expect("Failed to get initial exporter secret");

        // Perform self update
        let _update_result = creator_nostr_mls
            .self_update(group_id)
            .expect("Failed to perform self update");

        // Get the new exporter secret
        let final_secret = creator_nostr_mls
            .exporter_secret(group_id)
            .expect("Failed to get final exporter secret");

        // Verify the exporter secret has been rotated
        assert_ne!(
            initial_secret.secret, final_secret.secret,
            "Exporter secret should be different after self update"
        );

        // Verify the epoch has advanced
        assert!(
            final_secret.epoch > initial_secret.epoch,
            "Epoch should advance after self update (initial: {}, final: {})",
            initial_secret.epoch,
            final_secret.epoch
        );

        // Verify the group ID remains the same
        assert_eq!(
            initial_secret.mls_group_id, final_secret.mls_group_id,
            "Group ID should remain the same"
        );
    }
}
