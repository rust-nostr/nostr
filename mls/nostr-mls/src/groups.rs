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

use nostr::prelude::*;
use nostr_mls_storage::groups::types as group_types;
use nostr_mls_storage::messages::types as message_types;
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
pub struct GroupResult {
    /// The stored group
    pub group: group_types::Group,
    /// A vec of Kind:444 Welcome Events to be published for members added during creation.
    pub welcome_rumors: Vec<UnsignedEvent>,
}

/// Result of updating a group
#[derive(Debug)]
pub struct UpdateGroupResult {
    /// A Kind:445 Event containing the proposal or commit message. To be published to the group relays.
    pub evolution_event: Event,
    /// A vec of Kind:444 Welcome Events to be published for any members added as part of the update.
    pub welcome_rumors: Option<Vec<UnsignedEvent>>,
}

/// Configuration data for the Group
#[derive(Debug, Clone)]
pub struct NostrGroupConfigData {
    /// Group name
    pub name: String,
    /// Group description
    pub description: String,
    /// URL to encrypted group image
    pub image_url: Option<String>,
    /// Key to decrypt the image
    pub image_key: Option<Vec<u8>>,
    /// Relays used by the group
    pub relays: Vec<RelayUrl>,
    /// Group admins
    pub admins: Vec<PublicKey>,
}

/// Configuration for updating group data with optional fields
#[derive(Debug, Clone)]
pub struct NostrGroupDataUpdate {
    /// Group name (optional)
    pub name: Option<String>,
    /// Group description (optional)
    pub description: Option<String>,
    /// URL to encrypted group image (optional, use Some(None) to clear)
    pub image_url: Option<Option<String>>,
    /// Key to decrypt the image (optional, use Some(None) to clear)
    pub image_key: Option<Option<Vec<u8>>>,
    /// Relays used by the group (optional)
    pub relays: Option<Vec<RelayUrl>>,
    /// Group admins (optional)
    pub admins: Option<Vec<PublicKey>>,
}

impl NostrGroupConfigData {
    /// Creates NostrGroupConfigData
    pub fn new(
        name: String,
        description: String,
        image_url: Option<String>,
        image_key: Option<Vec<u8>>,
        relays: Vec<RelayUrl>,
        admins: Vec<PublicKey>,
    ) -> Self {
        Self {
            name,
            description,
            image_url,
            image_key,
            relays,
            admins,
        }
    }
}

impl NostrGroupDataUpdate {
    /// Creates a new empty update configuration
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            image_url: None,
            image_key: None,
            relays: None,
            admins: None,
        }
    }

    /// Sets the name to be updated
    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description to be updated
    pub fn description<T: Into<String>>(mut self, description: T) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the image URL to be updated
    pub fn image_url<T: Into<String>>(mut self, image_url: Option<T>) -> Self {
        self.image_url = Some(image_url.map(Into::into));
        self
    }

    /// Sets the image key to be updated
    pub fn image_key(mut self, image_key: Option<Vec<u8>>) -> Self {
        self.image_key = Some(image_key);
        self
    }

    /// Sets the relays to be updated
    pub fn relays(mut self, relays: Vec<RelayUrl>) -> Self {
        self.relays = Some(relays);
        self
    }

    /// Sets the admins to be updated
    pub fn admins(mut self, admins: Vec<PublicKey>) -> Self {
        self.admins = Some(admins);
        self
    }
}

impl Default for NostrGroupDataUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl<Storage> NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
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
        let own_leaf = group.own_leaf().ok_or(Error::OwnLeafNotFound)?;
        let credentials: BasicCredential =
            BasicCredential::try_from(own_leaf.credential().clone())?;
        let hex_bytes: &[u8] = credentials.identity();
        let hex_str: &str = str::from_utf8(hex_bytes)?;
        let public_key = PublicKey::from_hex(hex_str)?;
        Ok(public_key)
    }

    /// Checks if the LeafNode is an admin of an MLS group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `leaf_node` - The leaf to check as an admin
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - The leaf node is an admin
    /// * `Ok(false)` - The leaf node is not an admin
    /// * `Err(Error)` - If the public key cannot be extracted or the group is not found
    pub(crate) fn is_leaf_node_admin(
        &self,
        group_id: &GroupId,
        leaf_node: &LeafNode,
    ) -> Result<bool, Error> {
        let pubkey = self.pubkey_for_leaf_node(leaf_node)?;
        let stored_group = self.get_group(group_id)?.ok_or(Error::GroupNotFound)?;
        Ok(stored_group.admin_pubkeys.contains(&pubkey))
    }

    /// Checks if the Member is an admin of an MLS group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `member` - The member to check as an admin
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - The member is an admin
    /// * `Ok(false)` - The member is not an admin
    /// * `Err(Error)` - If the public key cannot be extracted or the group is not found
    pub(crate) fn is_member_admin(
        &self,
        group_id: &GroupId,
        member: &Member,
    ) -> Result<bool, Error> {
        let pubkey = self.pubkey_for_member(member)?;
        let stored_group = self.get_group(group_id)?.ok_or(Error::GroupNotFound)?;
        Ok(stored_group.admin_pubkeys.contains(&pubkey))
    }

    /// Extracts the public key from a leaf node
    ///
    /// # Arguments
    ///
    /// * `leaf_node` - Reference to the leaf node
    ///
    /// # Returns
    ///
    /// * `Ok(PublicKey)` - The public key extracted from the leaf node
    /// * `Err(Error)` - If the public key cannot be extracted or there is an error converting the public key to hex
    pub(crate) fn pubkey_for_leaf_node(&self, leaf_node: &LeafNode) -> Result<PublicKey, Error> {
        let credentials: BasicCredential =
            BasicCredential::try_from(leaf_node.credential().clone())?;
        let hex_bytes: &[u8] = credentials.identity();
        let hex_str: &str = str::from_utf8(hex_bytes)?;
        let public_key = PublicKey::from_hex(hex_str)?;
        Ok(public_key)
    }

    /// Extracts the public key from a member
    ///
    /// # Arguments
    ///
    /// * `member` - Reference to the member
    ///
    /// # Returns
    ///
    /// * `Ok(PublicKey)` - The public key extracted from the member
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
        let own_leaf: &LeafNode = group.own_leaf().ok_or(Error::OwnLeafNotFound)?;
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
    /// * `group_id` - The MLS group ID to load
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MlsGroup))` - The loaded group if found
    /// * `Ok(None)` - If no group exists with the given ID
    /// * `Err(Error)` - If there is an error loading the group
    pub(crate) fn load_mls_group(&self, group_id: &GroupId) -> Result<Option<MlsGroup>, Error> {
        MlsGroup::load(self.provider.storage(), group_id)
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
    pub(crate) fn exporter_secret(
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
                    .export_secret(self.provider.crypto(), "nostr", b"nostr", 32)?
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

    /// Gets the public keys of members that will be added from pending proposals in an MLS group
    ///
    /// This helper method loads an MLS group and examines its pending proposals to identify
    /// any Add proposals that would add new members to the group. For each new member,
    /// it extracts their public key from their LeafNode.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID to examine for pending proposals
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PublicKey>)` - List of public keys for newly added members in pending proposals
    /// * `Err(Error)` - If there's an error loading the group or extracting member information
    pub(crate) fn pending_added_members_pubkeys(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<PublicKey>, Error> {
        // Load the MLS group
        let mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let mut added_pubkeys = Vec::new();

        // Get pending proposals from the group
        let pending_proposals = mls_group.pending_proposals();

        // Extract public keys from Add proposals
        for proposal in pending_proposals {
            if let Proposal::Add(add_proposal) = proposal.proposal() {
                // Extract the public key from the LeafNode using the same pattern as other methods
                let leaf_node = add_proposal.key_package().leaf_node();
                let pubkey = self.pubkey_for_leaf_node(leaf_node)?;
                added_pubkeys.push(pubkey);
            }
        }

        Ok(added_pubkeys)
    }

    /// Add members to a group
    ///
    /// NOTE: This function doesn't merge the pending commit. Clients must call this function manually only after successful publish of the commit message to relays.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `key_package_events` - The nostr key package events (Kind:443) for each new member to add
    ///
    /// # Returns
    ///
    /// * `Ok(UpdateGroupResult)`
    /// * `Err(Error)` - If there is an error adding members
    pub fn add_members(
        &self,
        group_id: &GroupId,
        key_package_events: &[Event],
    ) -> Result<UpdateGroupResult, Error> {
        let mut mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;
        let mls_signer: SignatureKeyPair = self.load_mls_signer(&mls_group)?;

        // Check if current user is an admin
        let own_leaf = mls_group.own_leaf().ok_or(Error::OwnLeafNotFound)?;
        if !self.is_leaf_node_admin(mls_group.group_id(), own_leaf)? {
            return Err(Error::Group(
                "Only group admins can add members".to_string(),
            ));
        }

        // Parse key packages from events
        let mut key_packages_vec: Vec<KeyPackage> = Vec::new();
        for event in key_package_events {
            // TODO: Error handling for failure here
            let key_package: KeyPackage = self.parse_key_package(event)?;
            key_packages_vec.push(key_package);
        }

        let (commit_message, welcome_message, _group_info) = mls_group
            .add_members(&self.provider, &mls_signer, &key_packages_vec)
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_commit_message = commit_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        let commit_event =
            self.build_encrypted_message_event(mls_group.group_id(), serialized_commit_message)?;

        // Create processed_message to track state of message
        let processed_message: message_types::ProcessedMessage = message_types::ProcessedMessage {
            wrapper_event_id: commit_event.id,
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::ProcessedCommit,
            failure_reason: None,
        };

        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;

        let serialized_welcome_message = welcome_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        // Get relays for this group
        let group_relays = self
            .get_relays(mls_group.group_id())?
            .into_iter()
            .collect::<Vec<_>>();

        let welcome_rumors = self.build_welcome_rumors_for_key_packages(
            &mls_group,
            serialized_welcome_message,
            key_package_events.to_vec(),
            &group_relays,
        )?;

        // let serialized_group_info = group_info
        //     .map(|g| {
        //         g.tls_serialize_detached()
        //             .map_err(|e| Error::Group(e.to_string()))
        //     })
        //     .transpose()?;

        Ok(UpdateGroupResult {
            evolution_event: commit_event,
            welcome_rumors, // serialized_group_info,
        })
    }

    /// Remove members from a group
    ///
    /// NOTE: This function doesn't merge the pending commit. Clients must call this function manually only after successful publish of the commit message to relays.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `pubkeys` - The Nostr public keys of the members to remove
    ///
    /// # Returns
    ///
    /// * `Ok(UpdateGroupResult)`
    /// * `Err(Error)` - If there is an error removing members
    pub fn remove_members(
        &self,
        group_id: &GroupId,
        pubkeys: &[PublicKey],
    ) -> Result<UpdateGroupResult, Error> {
        let mut mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let signer: SignatureKeyPair = self.load_mls_signer(&mls_group)?;

        // Check if current user is an admin
        let own_leaf = mls_group.own_leaf().ok_or(Error::OwnLeafNotFound)?;
        if !self.is_leaf_node_admin(group_id, own_leaf)? {
            return Err(Error::Group(
                "Only group admins can remove members".to_string(),
            ));
        }

        // Convert pubkeys to leaf indices
        let mut leaf_indices = Vec::new();
        let members = mls_group.members();

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

        // TODO: Get a list of users to be added from any proposals and create welcome events for them

        let (commit_message, welcome_option, _group_info) = mls_group
            .remove_members(&self.provider, &signer, &leaf_indices)
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_commit_message = commit_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        let commit_event =
            self.build_encrypted_message_event(mls_group.group_id(), serialized_commit_message)?;

        // Create processed_message to track state of message
        let processed_message: message_types::ProcessedMessage = message_types::ProcessedMessage {
            wrapper_event_id: commit_event.id,
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::ProcessedCommit,
            failure_reason: None,
        };

        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;

        // For now, if we find welcomes, throw an error.
        if welcome_option.is_some() {
            return Err(Error::Group(
                "Found welcomes when removing users".to_string(),
            ));
        }
        // let serialized_welcome_message = welcome_option
        //     .map(|w| {
        //         w.tls_serialize_detached()
        //             .map_err(|e| Error::Group(e.to_string()))
        //     })
        //     .transpose()?;

        // let serialized_group_info = group_info
        //     .map(|g| {
        //         g.tls_serialize_detached()
        //             .map_err(|e| Error::Group(e.to_string()))
        //     })
        //     .transpose()?;

        Ok(UpdateGroupResult {
            evolution_event: commit_event,
            welcome_rumors: None, // serialized_group_info,
        })
    }

    fn update_group_data_extension(
        &self,
        mls_group: &mut MlsGroup,
        group_id: &GroupId,
        group_data: &NostrGroupDataExtension,
    ) -> Result<UpdateGroupResult, Error> {
        // Check if current user is an admin
        let own_leaf = mls_group.own_leaf().ok_or(Error::OwnLeafNotFound)?;
        if !self.is_leaf_node_admin(group_id, own_leaf)? {
            return Err(Error::Group(
                "Only group admins can update group context extensions".to_string(),
            ));
        }

        let extension = Self::get_unknown_extension_from_group_data(group_data)?;
        let mut extensions = mls_group.extensions().clone();
        extensions.add_or_replace(extension);

        let signature_keypair = self.load_mls_signer(mls_group)?;
        let (message_out, _, _) = mls_group.update_group_context_extensions(
            &self.provider,
            extensions,
            &signature_keypair,
        )?;
        let commit_event = self.build_encrypted_message_event(
            mls_group.group_id(),
            message_out.tls_serialize_detached()?,
        )?;

        // Create processed_message to track state of message
        let processed_message: message_types::ProcessedMessage = message_types::ProcessedMessage {
            wrapper_event_id: commit_event.id,
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::ProcessedCommit,
            failure_reason: None,
        };

        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;

        Ok(UpdateGroupResult {
            evolution_event: commit_event,
            welcome_rumors: None,
        })
    }

    /// Updates group data with the specified configuration
    ///
    /// This method allows updating one or more fields of the group data in a single operation.
    /// Only the fields specified in the update configuration will be modified.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `update` - Configuration specifying which fields to update and their new values
    ///
    /// # Returns
    ///
    /// * `Ok(UpdateGroupResult)` - Update result containing the evolution event
    /// * `Err(Error)` - If the group is not found or the operation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Update only the name
    /// let update = NostrGroupDataUpdate::new().name("New Group Name");
    /// mls.update_group_data(&group_id, update)?;
    ///
    /// // Update name and description together
    /// let update = NostrGroupDataUpdate::new()
    ///     .name("New Name")
    ///     .description("New Description");
    /// mls.update_group_data(&group_id, update)?;
    ///
    /// // Update image, clearing the existing one
    /// let update = NostrGroupDataUpdate::new().image_url(None);
    /// mls.update_group_data(&group_id, update)?;
    /// ```
    pub fn update_group_data(
        &self,
        group_id: &GroupId,
        update: NostrGroupDataUpdate,
    ) -> Result<UpdateGroupResult, Error> {
        let mut mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let mut group_data = NostrGroupDataExtension::from_group(&mls_group)?;

        // Apply updates only for fields that are specified
        if let Some(name) = update.name {
            group_data.name = name;
        }

        if let Some(description) = update.description {
            group_data.description = description;
        }

        if let Some(image_url) = update.image_url {
            group_data.image_url = image_url;
        }

        if let Some(image_key) = update.image_key {
            group_data.image_key = image_key;
        }

        if let Some(relays) = update.relays {
            group_data.relays = relays.into_iter().collect();
        }

        if let Some(admins) = update.admins {
            group_data.admins = admins.into_iter().collect();
        }

        self.update_group_data_extension(&mut mls_group, group_id, &group_data)
    }

    /// Retrieves the set of relay URLs associated with an MLS group
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    ///
    /// # Returns
    ///
    /// * `Ok(BTreeSet<RelayUrl>)` - Set of relay URLs where group messages are published
    /// * `Err(Error)` - If there is an error accessing storage or the group is not found
    pub fn get_relays(&self, group_id: &GroupId) -> Result<BTreeSet<RelayUrl>, Error> {
        let relays = self
            .storage()
            .group_relays(group_id)
            .map_err(|e| Error::Group(e.to_string()))?;
        Ok(relays.into_iter().map(|r| r.relay_url).collect())
    }

    fn get_unknown_extension_from_group_data(
        group_data: &NostrGroupDataExtension,
    ) -> Result<Extension, Error> {
        let serialized_group_data = group_data.as_raw().tls_serialize_detached()?;

        Ok(Extension::Unknown(
            group_data.extension_type(),
            UnknownExtension(serialized_group_data),
        ))
    }

    /// Creates a new MLS group with the specified members and settings.
    ///
    /// This function creates a new MLS group with the given name, description, members, and administrators.
    /// It generates the necessary cryptographic credentials, configures the group with Nostr-specific extensions,
    /// and adds the specified members.
    ///
    /// NOTE: This function doesn't merge the pending commit. Clients must call this function manually only after successful publish of the commit message to relays.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group
    /// * `description` - A description of the group
    /// * `creator_public_key` - The Nostr public key of the group creator
    /// * `member_key_package_events` - A vector of Nostr events (Kind:443) containing key packages for the initial group members
    /// * `admins` - A vector of Nostr public keys for group administrators
    /// * `group_relays` - A vector of relay URLs where group messages will be published
    ///
    /// # Returns
    ///
    /// A `GroupResult` containing:
    /// - The created MLS group
    /// - A Vec of UnsignedEvents representing the welcomes to be sent to new users
    ///
    /// # Errors
    ///
    /// Returns a `Error` if:
    /// - Credential generation fails
    /// - Group creation fails
    /// - Adding members fails
    /// - Message serialization fails
    pub fn create_group(
        &self,
        creator_public_key: &PublicKey,
        member_key_package_events: Vec<Event>,
        config: NostrGroupConfigData,
    ) -> Result<GroupResult, Error> {
        // Get member pubkeys
        let member_pubkeys = member_key_package_events
            .clone()
            .into_iter()
            .map(|e| e.pubkey)
            .collect::<Vec<PublicKey>>();

        let admins = config.admins.clone();

        // Validate group members
        self.validate_group_members(creator_public_key, &member_pubkeys, &admins)?;

        let (credential, signer) = self.generate_credential_with_key(creator_public_key)?;

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Credential and signer created, {:?}",
            credential
        );

        let group_data = NostrGroupDataExtension::new(
            config.name,
            config.description,
            admins,
            config.relays.clone(),
            config.image_url.clone(),
            config.image_key.clone(),
        );

        tracing::debug!(
            target: "nostr_mls::groups::create_mls_group",
            "Group data created, {:?}",
            group_data
        );

        let extension = Self::get_unknown_extension_from_group_data(&group_data)?;
        let required_capabilities_extension = self.required_capabilities_extension();
        let extensions = Extensions::from_vec(vec![extension, required_capabilities_extension])?;

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

        let mut key_packages_vec: Vec<KeyPackage> = Vec::new();
        for event in &member_key_package_events {
            // TODO: Error handling for failure here
            let key_package: KeyPackage = self.parse_key_package(event)?;
            key_packages_vec.push(key_package);
        }

        // Add members to the group
        let (_, welcome_out, _group_info) =
            mls_group.add_members(&self.provider, &signer, &key_packages_vec)?;

        // Merge the pending commit to finalize the group state - we do this during creation because we don't have a commit event to fan out to the group relays
        mls_group.merge_pending_commit(&self.provider)?;

        // Serialize the welcome message and send it to the members
        let serialized_welcome_message = welcome_out.tls_serialize_detached()?;

        let welcome_rumors = self
            .build_welcome_rumors_for_key_packages(
                &mls_group,
                serialized_welcome_message,
                member_key_package_events,
                &config.relays,
            )?
            .ok_or(Error::Welcome("Error creating welcome rumors".to_string()))?;

        // Save the NostrMLS Group
        let group = group_types::Group {
            mls_group_id: mls_group.group_id().clone(),
            nostr_group_id: group_data.clone().nostr_group_id,
            name: group_data.clone().name,
            description: group_data.clone().description,
            admin_pubkeys: group_data.clone().admins,
            last_message_id: None,
            last_message_at: None,
            epoch: mls_group.epoch().as_u64(),
            state: group_types::GroupState::Active,
            image_url: config.image_url,
            image_key: config.image_key,
        };

        self.storage().save_group(group.clone()).map_err(
            |e: nostr_mls_storage::groups::error::GroupError| Error::Group(e.to_string()),
        )?;

        // Always (re-)save the group relays after saving the group
        for relay_url in config.relays.into_iter() {
            let group_relay = group_types::GroupRelay {
                mls_group_id: group.mls_group_id.clone(),
                relay_url,
            };

            self.storage()
                .save_group_relay(group_relay)
                .map_err(|e| Error::Group(e.to_string()))?;
        }

        Ok(GroupResult {
            group,
            welcome_rumors,
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
    /// NOTE: This function doesn't merge the pending commit. Clients must call this function manually only after successful publish of the commit message to relays.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The ID of the MLS group
    ///
    /// # Returns
    ///
    /// An UpdateGroupResult
    ///
    /// # Errors
    ///
    /// Returns a Error if:
    /// - The group cannot be loaded from storage
    /// - The specified group is not found
    /// - Failed to generate or store signature keypair
    /// - Failed to perform self-update operation
    pub fn self_update(&self, group_id: &GroupId) -> Result<UpdateGroupResult, Error> {
        let mut mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let current_secret: group_types::GroupExporterSecret = self
            .storage()
            .get_group_exporter_secret(group_id, mls_group.epoch().as_u64())
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupExporterSecretNotFound)?;

        tracing::debug!(target: "nostr_openmls::groups::self_update", "Current epoch: {:?}", current_secret.epoch);

        // Load current signer
        let current_signer: SignatureKeyPair = self.load_mls_signer(&mls_group)?;

        // Get own leaf
        let own_leaf = mls_group.own_leaf().ok_or(Error::OwnLeafNotFound)?;

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

        let new_signer_bundle = NewSignerBundle {
            signer: &new_signature_keypair,
            credential_with_key: new_credential_with_key.clone(),
        };

        let leaf_node_params = LeafNodeParameters::builder()
            .with_credential_with_key(new_credential_with_key)
            .with_capabilities(own_leaf.capabilities().clone())
            .with_extensions(own_leaf.extensions().clone())
            .build();

        let commit_message_bundle = mls_group.self_update_with_new_signer(
            &self.provider,
            &current_signer,
            new_signer_bundle,
            leaf_node_params,
        )?;

        // Serialize the message
        let serialized_commit_message = commit_message_bundle.commit().tls_serialize_detached()?;

        let commit_event =
            self.build_encrypted_message_event(mls_group.group_id(), serialized_commit_message)?;

        // Create processed_message to track state of message
        let processed_message: message_types::ProcessedMessage = message_types::ProcessedMessage {
            wrapper_event_id: commit_event.id,
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::ProcessedCommit,
            failure_reason: None,
        };

        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;

        let serialized_welcome_message = commit_message_bundle
            .welcome()
            .map(|w| {
                w.tls_serialize_detached()
                    .map_err(|e| Error::Group(e.to_string()))
            })
            .transpose()?;

        // For now, if we find welcomes, throw an error.
        if serialized_welcome_message.is_some() {
            return Err(Error::Group(
                "Found welcomes when performing a self update".to_string(),
            ));
        }

        Ok(UpdateGroupResult {
            evolution_event: commit_event,
            welcome_rumors: None, // serialized_group_info,
        })
    }

    /// Create a proposal to leave the group
    /// It's not possible to unilaterally leave a group because you can't commit yourself out of the tree.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The ID of the MLS group
    ///
    /// # Returns
    /// * `Ok(UpdateGroupResult)`
    pub fn leave_group(&self, group_id: &GroupId) -> Result<UpdateGroupResult, Error> {
        let mut group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;

        let signer: SignatureKeyPair = self.load_mls_signer(&group)?;

        let leave_message = group
            .leave_group(&self.provider, &signer)
            .map_err(|e| Error::Group(e.to_string()))?;

        let serialized_message_out = leave_message
            .tls_serialize_detached()
            .map_err(|e| Error::Group(e.to_string()))?;

        let evolution_event =
            self.build_encrypted_message_event(group.group_id(), serialized_message_out)?;

        // Create processed_message to track state of message
        let processed_message: message_types::ProcessedMessage = message_types::ProcessedMessage {
            wrapper_event_id: evolution_event.id,
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::ProcessedCommit,
            failure_reason: None,
        };

        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;

        Ok(UpdateGroupResult {
            evolution_event,
            welcome_rumors: None,
        })
    }

    /// Merge any pending commits.
    /// This should be called AFTER publishing the Kind:445 message that contains a commit message to mitigate race conditions
    ///
    /// # Arguments
    /// * `group_id` - the MlsGroup GroupId value
    ///
    /// Returns
    /// * `Ok(())` - if the commits were merged successfully
    /// * Err(GroupError) - if something goes wrong
    pub fn merge_pending_commit(&self, group_id: &GroupId) -> Result<(), Error> {
        let mut mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;
        mls_group.merge_pending_commit(&self.provider)?;

        // Sync the stored group metadata with the updated MLS group state
        self.sync_group_metadata_from_mls(group_id)?;

        Ok(())
    }

    /// Synchronizes the stored group metadata with the current MLS group state
    ///
    /// This helper method ensures that all fields in the stored `group_types::Group`
    /// remain consistent with the MLS group state and extensions after operations.
    /// It should be called after any operation that changes the group state or extensions.
    ///
    /// # Arguments
    /// * `group_id` - The MLS group ID to synchronize
    ///
    /// # Returns
    /// * `Ok(())` - if synchronization succeeds
    /// * `Err(Error)` - if the group is not found or synchronization fails
    pub fn sync_group_metadata_from_mls(&self, group_id: &GroupId) -> Result<(), Error> {
        let mls_group = self.load_mls_group(group_id)?.ok_or(Error::GroupNotFound)?;
        let mut stored_group = self.get_group(group_id)?.ok_or(Error::GroupNotFound)?;

        // Update epoch from MLS group
        stored_group.epoch = mls_group.epoch().as_u64();

        // Update extension data from NostrGroupDataExtension
        if let Ok(group_data) = NostrGroupDataExtension::from_group(&mls_group) {
            stored_group.name = group_data.name;
            stored_group.description = group_data.description;
            stored_group.image_url = group_data.image_url;
            stored_group.image_key = group_data.image_key;
            stored_group.admin_pubkeys = group_data.admins;
            stored_group.nostr_group_id = group_data.nostr_group_id;
        }

        self.storage()
            .save_group(stored_group)
            .map_err(|e| Error::Group(e.to_string()))?;

        Ok(())
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

    /// Creates a NIP-44 encrypted message event Kind: 445 signing with an ephemeral keypair.
    pub(crate) fn build_encrypted_message_event(
        &self,
        group_id: &GroupId,
        serialized_content: Vec<u8>,
    ) -> Result<Event, Error> {
        let group = self.get_group(group_id)?.ok_or(Error::GroupNotFound)?;

        // Export secret
        let secret: group_types::GroupExporterSecret = self.exporter_secret(group_id)?;

        // Convert that secret to nostr keys
        let secret_key: SecretKey = SecretKey::from_slice(&secret.secret)?;
        let export_nostr_keys: Keys = Keys::new(secret_key);

        // Encrypt the message content
        // At some group size this will become too large for NIP44 encryption or relay event size limits.
        // We're not sure yet what size, but it's something to be aware of.
        let encrypted_content: String = nip44::encrypt(
            export_nostr_keys.secret_key(),
            &export_nostr_keys.public_key,
            &serialized_content,
            nip44::Version::default(),
        )?;

        // Generate ephemeral key
        let ephemeral_nostr_keys: Keys = Keys::generate();

        let tag: Tag = Tag::custom(TagKind::h(), [hex::encode(group.nostr_group_id)]);

        let event = EventBuilder::new(Kind::MlsGroupMessage, encrypted_content)
            .tag(tag)
            .sign_with_keys(&ephemeral_nostr_keys)?;

        Ok(event)
    }

    pub(crate) fn build_welcome_rumors_for_key_packages(
        &self,
        group: &MlsGroup,
        serialized_welcome: Vec<u8>,
        key_package_events: Vec<Event>,
        group_relays: &[RelayUrl],
    ) -> Result<Option<Vec<UnsignedEvent>>, Error> {
        let committer_pubkey = self.get_own_pubkey(group)?;
        let mut welcome_rumors_vec = Vec::new();

        for event in key_package_events {
            // Build welcome event rumors for each new user
            let welcome_rumor =
                EventBuilder::new(Kind::MlsWelcome, hex::encode(&serialized_welcome))
                    .tags(vec![
                        Tag::from_standardized(TagStandard::Relays(group_relays.to_vec())),
                        Tag::event(event.id),
                    ])
                    .build(committer_pubkey);

            welcome_rumors_vec.push(welcome_rumor);
        }

        let welcome_rumors = if !welcome_rumors_vec.is_empty() {
            Some(welcome_rumors_vec)
        } else {
            None
        };

        Ok(welcome_rumors)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use nostr::{Keys, PublicKey};
    use nostr_mls_memory_storage::NostrMlsMemoryStorage;
    use nostr_mls_storage::messages::{types as message_types, MessageStorage};
    use openmls::group::GroupId;
    use openmls::prelude::BasicCredential;

    use super::NostrGroupDataExtension;
    use crate::groups::NostrGroupDataUpdate;
    use crate::test_util::*;
    use crate::tests::create_test_nostr_mls;

    #[test]
    fn test_validate_group_members() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();
        let member_pks: Vec<PublicKey> = members.iter().map(|k| k.public_key()).collect();

        // Test valid configuration
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &member_pks, &admins)
            .is_ok());

        // Test creator not in admin list
        let bad_admins = vec![member_pks[0]];
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &member_pks, &bad_admins)
            .is_err());

        // Test creator in member list
        let bad_members = vec![creator_pk, member_pks[0]];
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &bad_members, &admins)
            .is_err());

        // Test admin not in member list
        let non_member = Keys::generate().public_key();
        let bad_admins = vec![creator_pk, non_member];
        assert!(nostr_mls
            .validate_group_members(&creator_pk, &member_pks, &bad_admins)
            .is_err());
    }

    #[test]
    fn test_create_group_basic() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Verify group was created with correct members
        let members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get members");

        assert_eq!(members.len(), 3); // creator + 2 initial members
        assert!(members.contains(&creator_pk));
        for member_keys in &initial_members {
            assert!(members.contains(&member_keys.public_key()));
        }
    }

    #[test]
    fn test_get_members() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Test get_members
        let members = creator_nostr_mls
            .get_members(group_id)
            .expect("Failed to get members");

        assert_eq!(members.len(), 3); // creator + 2 initial members
        assert!(members.contains(&creator_pk));
        for member_keys in &initial_members {
            assert!(members.contains(&member_keys.public_key()));
        }
    }

    #[test]
    fn test_add_members_epoch_advancement() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the initial group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Get initial epoch
        let initial_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get group")
            .expect("Group should exist");
        let initial_epoch = initial_group.epoch;

        // Create key package event for new member
        let new_member = Keys::generate();
        let new_key_package_event = create_key_package_event(&creator_nostr_mls, &new_member);

        // Add the new member
        let _add_result = creator_nostr_mls
            .add_members(group_id, &[new_key_package_event])
            .expect("Failed to add member");

        // Merge the pending commit for the member addition
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for member addition");

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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

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
    fn test_admin_check() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
    create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Test admin check - verify creator is in admin list
        let stored_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get group")
            .expect("Group should exist");

        assert!(
            stored_group.admin_pubkeys.contains(&creator_pk),
            "Creator should be admin"
        );
    }

    #[test]
    fn test_admin_permission_checks() {
        let admin_nostr_mls = create_test_nostr_mls();
        let non_admin_nostr_mls = create_test_nostr_mls();

        // Generate keys
        let admin_keys = Keys::generate();
        let non_admin_keys = Keys::generate();
        let member1_keys = Keys::generate();

        let admin_pk = admin_keys.public_key();
        let _non_admin_pk = non_admin_keys.public_key();
        let member1_pk = member1_keys.public_key();

        // Create key package events for initial members
        let non_admin_event = create_key_package_event(&admin_nostr_mls, &non_admin_keys);
        let member1_event = create_key_package_event(&admin_nostr_mls, &member1_keys);

        // Create group with admin as creator, non_admin and member1 as members
        // Only admin is an admin
        let create_result = admin_nostr_mls
            .create_group(
                &admin_pk,
                vec![non_admin_event.clone(), member1_event.clone()],
                create_nostr_group_config_data(vec![admin_pk]), // Only admin is an admin
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        admin_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Create a new member to add
        let new_member_keys = Keys::generate();
        let _new_member_pk = new_member_keys.public_key();
        let new_member_event = create_key_package_event(&non_admin_nostr_mls, &new_member_keys);

        // Test that admin can add members (should work)
        let add_result = admin_nostr_mls.add_members(group_id, &[new_member_event]);
        assert!(add_result.is_ok(), "Admin should be able to add members");

        // Merge the pending commit for the member addition
        admin_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for member addition");

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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

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
        for member_keys in &initial_members {
            assert!(
                found_pubkeys.contains(&member_keys.public_key()),
                "Should find member pubkey: {:?}",
                member_keys.public_key()
            );
        }
        assert_eq!(found_pubkeys.len(), 3, "Should have 3 members total");
    }

    // TODO: Fix remaining test cases that need to be updated to match new API

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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Get initial epoch
        let initial_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get group")
            .expect("Group should exist");
        let initial_epoch = initial_group.epoch;

        // Remove a member
        let member_to_remove = initial_members[0].public_key();
        let _remove_result = creator_nostr_mls
            .remove_members(group_id, &[member_to_remove])
            .expect("Failed to remove member");

        // Merge the pending commit for the member removal
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for member removal");

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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

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

        // Merge the pending commit for the self update
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for self update");

        // Verify the result contains the expected data
        assert!(
            !update_result.evolution_event.content.is_empty(),
            "Evolution event should not be empty"
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
        for initial_member_keys in &initial_members {
            assert!(
                final_members.contains(&initial_member_keys.public_key()),
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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Get initial signature key from the leaf node
        let initial_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let initial_own_leaf = initial_mls_group
            .own_leaf()
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

        // Merge the pending commit for the self update
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for self update");

        // Get the new signature key
        let final_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let final_own_leaf = final_mls_group
            .own_leaf()
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
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Get initial exporter secret
        let initial_secret = creator_nostr_mls
            .exporter_secret(group_id)
            .expect("Failed to get initial exporter secret");

        // Perform self update
        let _update_result = creator_nostr_mls
            .self_update(group_id)
            .expect("Failed to perform self update");

        // Merge the pending commit for the self update
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for self update");

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

    #[test]
    fn test_update_group_data() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Get initial group data for comparison
        let initial_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let initial_group_data = NostrGroupDataExtension::from_group(&initial_mls_group).unwrap();

        // Test 1: Update only the name
        let new_name = "Updated Name".to_string();
        let update = NostrGroupDataUpdate::new().name(new_name.clone());
        let update_result = creator_nostr_mls
            .update_group_data(group_id, update)
            .expect("Failed to update group name");

        assert!(!update_result.evolution_event.content.is_empty());
        assert!(update_result.welcome_rumors.is_none());

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        let updated_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let updated_group_data = NostrGroupDataExtension::from_group(&updated_mls_group).unwrap();

        assert_eq!(updated_group_data.name, new_name);
        assert_eq!(
            updated_group_data.description,
            initial_group_data.description
        );
        assert_eq!(updated_group_data.image_url, initial_group_data.image_url);

        // Test 2: Update multiple fields at once
        let new_description = "Updated Description".to_string();
        let new_image_url = "https://example.com/new-image.png".to_string();
        let new_image_key = vec![1, 2, 3, 4, 5];

        let update = NostrGroupDataUpdate::new()
            .description(new_description.clone())
            .image_url(Some(new_image_url.clone()))
            .image_key(Some(new_image_key.clone()));

        let update_result = creator_nostr_mls
            .update_group_data(group_id, update)
            .expect("Failed to update multiple fields");

        assert!(!update_result.evolution_event.content.is_empty());

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        let final_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let final_group_data = NostrGroupDataExtension::from_group(&final_mls_group).unwrap();

        assert_eq!(final_group_data.name, new_name); // Should remain from previous update
        assert_eq!(final_group_data.description, new_description);
        assert_eq!(final_group_data.image_url, Some(new_image_url));
        assert_eq!(final_group_data.image_key, Some(new_image_key));

        // Test 3: Clear optional fields
        let update = NostrGroupDataUpdate::new()
            .image_url::<String>(None)
            .image_key(None);

        let update_result = creator_nostr_mls
            .update_group_data(group_id, update)
            .expect("Failed to clear optional fields");

        assert!(!update_result.evolution_event.content.is_empty());

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        let cleared_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let cleared_group_data = NostrGroupDataExtension::from_group(&cleared_mls_group).unwrap();

        assert_eq!(cleared_group_data.name, new_name);
        assert_eq!(cleared_group_data.description, new_description);
        assert_eq!(cleared_group_data.image_url, None);
        assert_eq!(cleared_group_data.image_key, None);

        // Test 4: Empty update (should succeed but not change anything)
        let empty_update = NostrGroupDataUpdate::new();
        let update_result = creator_nostr_mls
            .update_group_data(group_id, empty_update)
            .expect("Failed to apply empty update");

        assert!(!update_result.evolution_event.content.is_empty());

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        let unchanged_mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");
        let unchanged_group_data =
            NostrGroupDataExtension::from_group(&unchanged_mls_group).unwrap();

        assert_eq!(unchanged_group_data.name, cleared_group_data.name);
        assert_eq!(
            unchanged_group_data.description,
            cleared_group_data.description
        );
        assert_eq!(unchanged_group_data.image_url, cleared_group_data.image_url);
        assert_eq!(unchanged_group_data.image_key, cleared_group_data.image_key);
    }

    #[test]
    fn test_sync_group_metadata_from_mls() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins.clone()),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Get initial stored group state
        let initial_stored_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get initial stored group")
            .expect("Stored group should exist");

        // Modify the MLS group directly (simulating state change without sync)
        let mut mls_group = creator_nostr_mls
            .load_mls_group(group_id)
            .expect("Failed to load MLS group")
            .expect("MLS group should exist");

        // Create a new group data extension with different values
        let mut new_group_data = NostrGroupDataExtension::from_group(&mls_group).unwrap();
        new_group_data.name = "Synchronized Name".to_string();
        new_group_data.description = "Synchronized Description".to_string();

        // Apply the extension update to MLS group (but not to stored group)
        let extension =
            super::NostrMls::<NostrMlsMemoryStorage>::get_unknown_extension_from_group_data(
                &new_group_data,
            )
            .unwrap();
        let mut extensions = mls_group.extensions().clone();
        extensions.add_or_replace(extension);

        let signature_keypair = creator_nostr_mls.load_mls_signer(&mls_group).unwrap();
        let (_message_out, _, _) = mls_group
            .update_group_context_extensions(
                &creator_nostr_mls.provider,
                extensions,
                &signature_keypair,
            )
            .unwrap();

        // Merge the pending commit to advance epoch
        mls_group
            .merge_pending_commit(&creator_nostr_mls.provider)
            .unwrap();

        // At this point, MLS group has changed but stored group is stale
        let stale_stored_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get stale stored group")
            .expect("Stored group should exist");

        // Verify stored group is stale
        assert_eq!(stale_stored_group.name, initial_stored_group.name);
        assert_eq!(
            stale_stored_group.description,
            initial_stored_group.description
        );
        assert_eq!(stale_stored_group.epoch, initial_stored_group.epoch);

        // Now test our sync function
        creator_nostr_mls
            .sync_group_metadata_from_mls(group_id)
            .expect("Failed to sync group metadata");

        // Verify stored group is now synchronized
        let synced_stored_group = creator_nostr_mls
            .get_group(group_id)
            .expect("Failed to get synced stored group")
            .expect("Stored group should exist");

        assert_eq!(synced_stored_group.name, "Synchronized Name");
        assert_eq!(synced_stored_group.description, "Synchronized Description");
        assert!(synced_stored_group.epoch > initial_stored_group.epoch);
        assert_eq!(
            synced_stored_group.admin_pubkeys,
            admins.into_iter().collect::<BTreeSet<_>>()
        );

        // Verify other fields remain unchanged
        assert_eq!(
            synced_stored_group.mls_group_id,
            initial_stored_group.mls_group_id
        );
        assert_eq!(
            synced_stored_group.last_message_id,
            initial_stored_group.last_message_id
        );
        assert_eq!(
            synced_stored_group.last_message_at,
            initial_stored_group.last_message_at
        );
        assert_eq!(synced_stored_group.state, initial_stored_group.state);
    }

    #[test]
    fn test_extension_updates_create_processed_messages() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Merge the pending commit to apply the member additions
        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit");

        // Test that each extension update creates a ProcessedMessage
        let test_cases = vec![
            ("update_group_name", "New Name"),
            ("update_group_description", "New Description"),
        ];

        for (operation, _value) in test_cases {
            let update_result = match operation {
                "update_group_name" => {
                    let update = NostrGroupDataUpdate::new().name("New Name".to_string());
                    creator_nostr_mls.update_group_data(group_id, update)
                }
                "update_group_description" => {
                    let update =
                        NostrGroupDataUpdate::new().description("New Description".to_string());
                    creator_nostr_mls.update_group_data(group_id, update)
                }
                _ => panic!("Unknown operation"),
            };

            let update_result = update_result.unwrap_or_else(|_| panic!("Failed to {}", operation));
            let commit_event_id = update_result.evolution_event.id;

            // Verify ProcessedMessage was created with correct state
            let processed_message = creator_nostr_mls
                .storage()
                .find_processed_message_by_event_id(&commit_event_id)
                .expect("Failed to query processed message")
                .expect("ProcessedMessage should exist");

            assert_eq!(processed_message.wrapper_event_id, commit_event_id);
            assert_eq!(processed_message.message_event_id, None);
            assert_eq!(
                processed_message.state,
                message_types::ProcessedMessageState::ProcessedCommit
            );
            assert_eq!(processed_message.failure_reason, None);

            // Clean up by merging the commit
            creator_nostr_mls
                .merge_pending_commit(group_id)
                .unwrap_or_else(|_| panic!("Failed to merge pending commit for {}", operation));
        }
    }

    #[test]
    fn test_stored_group_sync_after_all_operations() {
        let creator_nostr_mls = create_test_nostr_mls();
        let (creator, initial_members, admins) = create_test_group_members();
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in &initial_members {
            let key_package_event = create_key_package_event(&creator_nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = creator_nostr_mls
            .create_group(
                &creator_pk,
                initial_key_package_events,
                create_nostr_group_config_data(admins),
            )
            .expect("Failed to create group");

        let group_id = &create_result.group.mls_group_id;

        // Helper function to verify stored group epoch matches MLS group epoch
        let verify_epoch_sync = || {
            let mls_group = creator_nostr_mls.load_mls_group(group_id).unwrap().unwrap();
            let stored_group = creator_nostr_mls.get_group(group_id).unwrap().unwrap();
            assert_eq!(
                stored_group.epoch,
                mls_group.epoch().as_u64(),
                "Stored group epoch should match MLS group epoch"
            );
        };

        // Test 1: After group creation (should already be synced)
        verify_epoch_sync();

        // Test 2: After adding members
        let new_member = Keys::generate();
        let new_key_package_event = create_key_package_event(&creator_nostr_mls, &new_member);
        let _add_result = creator_nostr_mls
            .add_members(group_id, &[new_key_package_event])
            .expect("Failed to add member");

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for add member");
        verify_epoch_sync();

        // Test 3: After self update
        // Ensure the exporter secret exists before self update (this creates it if it doesn't exist)
        let _initial_secret = creator_nostr_mls
            .exporter_secret(group_id)
            .expect("Failed to get initial exporter secret");

        let _self_update_result = creator_nostr_mls
            .self_update(group_id)
            .expect("Failed to perform self update");

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for self update");
        verify_epoch_sync();

        // Test 4: After extension updates
        let update = NostrGroupDataUpdate::new().name("Final Name".to_string());
        let _name_result = creator_nostr_mls
            .update_group_data(group_id, update)
            .expect("Failed to update group name");

        creator_nostr_mls
            .merge_pending_commit(group_id)
            .expect("Failed to merge pending commit for name update");
        verify_epoch_sync();

        // Test 5: Verify stored group metadata matches extension data
        let final_mls_group = creator_nostr_mls.load_mls_group(group_id).unwrap().unwrap();
        let final_stored_group = creator_nostr_mls.get_group(group_id).unwrap().unwrap();
        let final_group_data = NostrGroupDataExtension::from_group(&final_mls_group).unwrap();

        assert_eq!(final_stored_group.name, final_group_data.name);
        assert_eq!(final_stored_group.description, final_group_data.description);
        assert_eq!(final_stored_group.admin_pubkeys, final_group_data.admins);
        assert_eq!(
            final_stored_group.nostr_group_id,
            final_group_data.nostr_group_id
        );
    }

    #[test]
    fn test_sync_group_metadata_error_cases() {
        let creator_nostr_mls = create_test_nostr_mls();

        // Test with non-existent group
        let non_existent_group_id = GroupId::from_slice(&[1, 2, 3, 4, 5]);
        let result = creator_nostr_mls.sync_group_metadata_from_mls(&non_existent_group_id);
        assert!(matches!(result, Err(crate::Error::GroupNotFound)));
    }
}
