//! Nostr MLS Messages
//!
//! This module provides functionality for creating, processing, and managing encrypted
//! messages in MLS groups. It handles:
//! - Message creation and encryption
//! - Message processing and decryption
//! - Message state tracking
//! - Integration with Nostr events
//!
//! Messages in Nostr MLS are wrapped in Nostr events (kind:445) for relay transmission.
//! The message content is encrypted using both MLS group keys and NIP-44 encryption.
//! Message state is tracked to handle processing status and failure scenarios.

use nostr::util::hex;
use nostr::{EventId, UnsignedEvent};
use nostr_mls_storage::NostrMlsStorageProvider;
use openmls::group::{GroupId, ValidationError};
use openmls_basic_credential::SignatureKeyPair;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use crate::error::Error;
use crate::prelude::*;
use crate::NostrMls;

/// Information about member changes in a commit message
#[derive(Debug, Clone)]
pub struct MemberChanges {
    /// Members that were added
    pub added_members: Vec<String>,
    /// Members that were removed
    pub removed_members: Vec<String>,
}

/// Result of processing an MLS message
#[derive(Debug)]
pub struct ProcessMessageResult {
    /// The decrypted message (for application messages)
    pub message: Option<UnsignedEvent>,
    /// Member changes (for commit messages)
    pub member_changes: Option<MemberChanges>,
    /// Commit message bytes (for self-remove proposals that get committed)
    pub commit: Option<Vec<u8>>,
    /// Welcome message bytes (for self-remove proposals that get committed)
    pub welcome: Option<Vec<u8>>,
}

/// Result of processing a Nostr event containing an MLS message
#[derive(Debug)]
pub struct ProcessedEventResult {
    /// The processed message (if any)
    pub message: Option<message_types::Message>,
    /// Member changes (for commit messages)
    pub member_changes: Option<MemberChanges>,
    /// Commit message bytes (for self-remove proposals that get committed)
    pub commit: Option<Vec<u8>>,
    /// Welcome message bytes (for self-remove proposals that get committed)
    pub welcome: Option<Vec<u8>>,
}

impl<Storage> NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    /// Retrieves a message by its Nostr event ID
    ///
    /// This function looks up a message in storage using its associated Nostr event ID.
    /// The message must have been previously processed and stored.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The Nostr event ID to look up
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Message))` - The message if found
    /// * `Ok(None)` - If no message exists with the given event ID
    /// * `Err(Error)` - If there is an error accessing storage
    pub fn get_message(&self, event_id: &EventId) -> Result<Option<message_types::Message>, Error> {
        self.storage()
            .find_message_by_event_id(event_id)
            .map_err(|e| Error::Message(e.to_string()))
    }

    /// Retrieves all messages for a specific MLS group
    ///
    /// This function returns all messages that have been processed and stored for a group,
    /// ordered by creation time.
    ///
    /// # Arguments
    ///
    /// * `mls_group_id` - The MLS group ID to get messages for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Message>)` - List of all messages for the group
    /// * `Err(Error)` - If there is an error accessing storage
    pub fn get_messages(
        &self,
        mls_group_id: &GroupId,
    ) -> Result<Vec<message_types::Message>, Error> {
        self.storage()
            .messages(mls_group_id)
            .map_err(|e| Error::Message(e.to_string()))
    }

    /// Creates an MLS-encrypted message from an unsigned Nostr event
    ///
    /// This internal function handles the MLS-level encryption of a message:
    /// 1. Loads the member's signing keys
    /// 2. Ensures the message has a unique ID
    /// 3. Serializes the message content
    /// 4. Creates and signs the MLS message
    ///
    /// # Arguments
    ///
    /// * `group` - The MLS group to create the message in
    /// * `rumor` - The unsigned Nostr event to encrypt
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The serialized encrypted MLS message
    /// * `Err(Error)` - If message creation or encryption fails
    pub fn create_message_for_event(
        &self,
        group: &mut MlsGroup,
        rumor: &mut UnsignedEvent,
    ) -> Result<Vec<u8>, Error> {
        // Load signer
        let signer: SignatureKeyPair = self.load_mls_signer(group)?;

        // Ensure rumor ID
        rumor.ensure_id();

        // Serialize as JSON
        let json: String = rumor.as_json();

        // Create message
        let message_out = group.create_message(&self.provider, &signer, json.as_bytes())?;

        let serialized_message = message_out.tls_serialize_detached()?;

        Ok(serialized_message)
    }

    /// Creates a complete encrypted Nostr event for an MLS group message
    ///
    /// This is the main entry point for creating group messages. The function:
    /// 1. Loads the MLS group and its metadata
    /// 2. Creates and encrypts the MLS message
    /// 3. Derives NIP-44 encryption keys from the group's secret
    /// 4. Creates a Nostr event wrapping the encrypted message
    /// 5. Stores the message state for tracking
    ///
    /// # Arguments
    ///
    /// * `mls_group_id` - The MLS group ID
    /// * `rumor` - The unsigned Nostr event to encrypt and send
    ///
    /// # Returns
    ///
    /// * `Ok(Event)` - The signed Nostr event ready for relay publication
    /// * `Err(Error)` - If message creation or encryption fails
    pub fn create_message(
        &self,
        mls_group_id: &GroupId,
        mut rumor: UnsignedEvent,
    ) -> Result<Event, Error> {
        // Load mls group
        let mut mls_group = self
            .load_mls_group(mls_group_id)?
            .ok_or(Error::GroupNotFound)?;

        // Load stored group
        let group: group_types::Group = self
            .get_group(mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupNotFound)?;

        // Create message
        let message: Vec<u8> = self.create_message_for_event(&mut mls_group, &mut rumor)?;

        // Export secret
        let secret: group_types::GroupExporterSecret = self.exporter_secret(mls_group_id)?;

        // Convert that secret to nostr keys
        let secret_key: SecretKey = SecretKey::from_slice(&secret.secret)?;
        let export_nostr_keys: Keys = Keys::new(secret_key);

        // Encrypt the message content
        let encrypted_content: String = nip44::encrypt(
            export_nostr_keys.secret_key(),
            &export_nostr_keys.public_key,
            &message,
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

    /// Creates an encrypted Nostr event for an MLS commit/proposal message
    ///
    /// This function handles the creation of commit and proposal messages for MLS groups.
    /// Unlike regular application messages, these are protocol-level messages that manage
    /// group membership and state changes.
    ///
    /// # Arguments
    ///
    /// * `mls_group_id` - The MLS group ID
    /// * `commit_proposal_message` - The serialized commit or proposal message bytes
    ///
    /// # Returns
    ///
    /// * `Ok(Event)` - The signed Nostr event ready for relay publication
    /// * `Err(Error)` - If message creation or encryption fails
    pub fn create_commit_proposal_message(
        &self,
        mls_group_id: &GroupId,
        commit_proposal_message: &[u8],
        secret_key: &[u8; 32],
    ) -> Result<Event, Error> {
        // Load stored group
        let group: group_types::Group = self
            .get_group(mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupNotFound)?;

        // Convert that secret to nostr keys
        let secret_key: SecretKey = SecretKey::from_slice(secret_key)?;
        let export_nostr_keys: Keys = Keys::new(secret_key);
        // Encrypt the message content
        let encrypted_content: String = nip44::encrypt(
            export_nostr_keys.secret_key(),
            &export_nostr_keys.public_key,
            &commit_proposal_message,
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

    /// Processes an incoming MLS message
    ///
    /// This internal function handles the MLS protocol-level message processing:
    /// 1. Deserializes the MLS message
    /// 2. Validates the message's group ID
    /// 3. Processes the message according to its type
    /// 4. Handles any resulting group state changes
    ///
    /// # Arguments
    ///
    /// * `group` - The MLS group the message belongs to
    /// * `message_bytes` - The serialized MLS message to process
    ///
    /// # Returns
    ///
    /// * `Ok(ProcessMessageResult)` - Contains the decrypted message and/or member changes
    /// * `Err(Error)` - If message processing fails
    pub fn process_message_for_group(
        &self,
        group: &mut MlsGroup,
        message_bytes: &[u8],
    ) -> Result<ProcessMessageResult, Error> {
        let mls_message = MlsMessageIn::tls_deserialize_exact(message_bytes)?;

        tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received message: {:?}", mls_message);
        let protocol_message = mls_message.try_into_protocol_message()?;

        // Return error if group ID doesn't match
        if protocol_message.group_id() != group.group_id() {
            return Err(Error::ProtocolGroupIdMismatch);
        }

        if protocol_message.content_type() == ContentType::Commit {
            tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "ABOUT TO PROCESS COMMIT MESSAGE");
        }

        let processed_message = match group.process_message(&self.provider, protocol_message) {
            Ok(processed_message) => processed_message,
            Err(ProcessMessageError::ValidationError(ValidationError::CannotDecryptOwnMessage)) => {
                return Err(Error::CannotDecryptOwnMessage);
            }
            Err(e) => {
                tracing::error!(target: "nostr_mls::messages::process_message_for_group", "Error processing message: {:?}", e);
                return Err(Error::Message(e.to_string()));
            }
        };

        tracing::debug!(
            target: "nostr_mls::messages::process_message_for_group",
            "Processed message: {:?}",
            processed_message
        );
        // Handle the processed message based on its type
        match processed_message.into_content() {
            ProcessedMessageContent::ApplicationMessage(application_message) => {
                // This is a message from a group member
                let bytes = application_message.into_bytes();
                let rumor: UnsignedEvent = UnsignedEvent::from_json(bytes)?;
                Ok(ProcessMessageResult {
                    message: Some(rumor),
                    member_changes: None,
                    commit: None,
                    welcome: None,
                })
            }
            ProcessedMessageContent::ProposalMessage(staged_proposal) => {
                // This is a proposal message
                tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received proposal message: {:?}", staged_proposal);
                // Check if this is a self-remove proposal
                if let Proposal::Remove(remove_proposal) = staged_proposal.proposal() {
                    if let Sender::Member(sender_leaf_index) = staged_proposal.sender().clone() {
                        let removed_index = remove_proposal.removed();
                        let is_self_remove = removed_index == sender_leaf_index;

                        if is_self_remove {
                            tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "This is a self-remove proposal");
                            let commit_result = self
                                .commit_proposal(group.group_id(), *staged_proposal)
                                .map_err(|e| Error::Group(e.to_string()))?;

                            // Get information about the removed member
                            let mut removed_members = Vec::new();
                            if let Some(member) = group.member_at(removed_index) {
                                if let Ok(credential) =
                                    BasicCredential::try_from(member.credential.clone())
                                {
                                    let identity_bytes = credential.identity();
                                    if let Ok(identity_str) = std::str::from_utf8(identity_bytes) {
                                        tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Self-removing member with identity: {}", identity_str);
                                        removed_members.push(identity_str.to_string());
                                    }
                                }
                            }

                            let member_changes = if !removed_members.is_empty() {
                                Some(MemberChanges {
                                    added_members: Vec::new(),
                                    removed_members,
                                })
                            } else {
                                None
                            };

                            return Ok(ProcessMessageResult {
                                message: None,
                                member_changes,
                                commit: Some(commit_result.commit_message),
                                welcome: commit_result.welcome_message,
                            });
                        } else {
                            tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "This is a remove proposal for another member");
                        }
                    }
                }

                Ok(ProcessMessageResult {
                    message: None,
                    member_changes: None,
                    commit: None,
                    welcome: None,
                })
            }
            ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
                // This is a commit message
                tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Received commit message");

                // Check proposals in the proposal queue to understand member changes
                let queued_proposals = staged_commit.queued_proposals();
                let mut added_members = Vec::new();
                let mut removed_members = Vec::new();

                for queued_proposal in queued_proposals {
                    match queued_proposal.proposal() {
                        Proposal::Add(add_proposal) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains Add proposal");
                            // Get information about the added member from add_proposal.key_package()
                            let key_package = add_proposal.key_package();
                            if let Ok(credential) = BasicCredential::try_from(
                                key_package.leaf_node().credential().clone(),
                            ) {
                                let identity_bytes = credential.identity();
                                if let Ok(identity_str) = std::str::from_utf8(identity_bytes) {
                                    tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Adding member with identity: {}", identity_str);
                                    added_members.push(identity_str.to_string());
                                }
                            }
                        }
                        Proposal::Remove(remove_proposal) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains Remove proposal");
                            let removed_index = remove_proposal.removed();
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Removing member at leaf index: {:?}", removed_index);

                            // Get information about the removed member through leaf index from group
                            if let Some(member) = group.member_at(removed_index) {
                                if let Ok(credential) =
                                    BasicCredential::try_from(member.credential.clone())
                                {
                                    let identity_bytes = credential.identity();
                                    if let Ok(identity_str) = std::str::from_utf8(identity_bytes) {
                                        tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Removing member with identity: {}", identity_str);
                                        removed_members.push(identity_str.to_string());
                                    }
                                }
                            }
                        }
                        Proposal::Update(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains Update proposal");
                        }
                        Proposal::PreSharedKey(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains PreSharedKey proposal");
                        }
                        Proposal::ReInit(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains ReInit proposal");
                        }
                        Proposal::ExternalInit(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains ExternalInit proposal");
                        }
                        Proposal::GroupContextExtensions(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains GroupContextExtensions proposal");
                        }
                        Proposal::AppAck(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains AppAck proposal");
                        }
                        Proposal::SelfRemove => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains SelfRemove proposal");
                        }
                        Proposal::Custom(_) => {
                            tracing::info!(target: "nostr_mls::messages::process_message_for_group", "Commit contains Custom proposal");
                        }
                    }
                }

                group
                    .merge_staged_commit(&self.provider, *staged_commit)
                    .map_err(|e| Error::Group(e.to_string()))?;
                group.merge_pending_commit(&self.provider)?;

                let member_changes = if !added_members.is_empty() || !removed_members.is_empty() {
                    Some(MemberChanges {
                        added_members,
                        removed_members,
                    })
                } else {
                    None
                };

                Ok(ProcessMessageResult {
                    message: None,
                    member_changes,
                    commit: None,
                    welcome: None,
                })
            }
            ProcessedMessageContent::ExternalJoinProposalMessage(external_join_proposal) => {
                tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received external join proposal message: {:?}", external_join_proposal);
                // TODO: Handle external join proposal
                Ok(ProcessMessageResult {
                    message: None,
                    member_changes: None,
                    commit: None,
                    welcome: None,
                })
            }
        }
    }

    /// Processes an incoming encrypted Nostr event containing an MLS message
    ///
    /// This is the main entry point for processing received messages. The function:
    /// 1. Validates the event kind
    /// 2. Loads the MLS group and its secret
    /// 3. Decrypts the NIP-44 encrypted content
    /// 4. Processes the MLS message
    /// 5. Updates message state in storage
    ///
    /// # Arguments
    ///
    /// * `event` - The received Nostr event
    ///
    /// # Returns
    ///
    /// * `Ok(ProcessedEventResult)` - Contains the processed message and member changes
    /// * `Err(Error)` - If message processing fails
    pub fn process_message(&self, event: &Event) -> Result<ProcessedEventResult, Error> {
        if event.kind != Kind::MlsGroupMessage {
            return Err(Error::UnexpectedEvent {
                expected: Kind::MlsGroupMessage,
                received: event.kind,
            });
        }

        let nostr_group_id_tag = event
            .tags
            .iter()
            .find(|tag| tag.kind() == TagKind::h())
            .ok_or(Error::Message("Group ID Tag not found".to_string()))?;

        let nostr_group_id: [u8; 32] = hex::decode(
            nostr_group_id_tag
                .content()
                .ok_or(Error::Message("Group ID Tag content not found".to_string()))?,
        )
        .map_err(|e| Error::Message(e.to_string()))?
        .try_into()
        .map_err(|_e| Error::Message("Failed to convert nostr group id to [u8; 32]".to_string()))?;

        let group = self
            .storage()
            .find_group_by_nostr_group_id(&nostr_group_id)
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupNotFound)?;

        // Load group exporter secret
        let secret: group_types::GroupExporterSecret = self
            .exporter_secret(&group.mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?;

        // Convert that secret to nostr keys
        let secret_key: SecretKey = SecretKey::from_slice(&secret.secret)?;
        let export_nostr_keys = Keys::new(secret_key);
        // Decrypt message
        let message_bytes: Vec<u8> = nip44::decrypt_to_bytes(
            export_nostr_keys.secret_key(),
            &export_nostr_keys.public_key,
            &event.content,
        )?;

        let mut mls_group = self
            .load_mls_group(&group.mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupNotFound)?;

        // The resulting serialized message is the MLS encrypted message that Bob sent
        // Now Bob can process the MLS message content and do what's needed with it
        match self.process_message_for_group(&mut mls_group, &message_bytes) {
            Ok(ProcessMessageResult {
                message: Some(mut rumor),
                member_changes,
                commit,
                welcome,
            }) => {
                let rumor_id: EventId = rumor.id();

                let message = message_types::Message {
                    id: rumor_id,
                    pubkey: rumor.pubkey,
                    kind: rumor.kind,
                    mls_group_id: group.mls_group_id.clone(),
                    created_at: rumor.created_at,
                    content: rumor.content.clone(),
                    tags: rumor.tags.clone(),
                    event: rumor.clone(),
                    wrapper_event_id: event.id,
                    state: message_types::MessageState::Processed,
                };

                tracing::debug!(target: "nostr_mls::messages::process_message", "Message: {:?}", message);
                Ok(ProcessedEventResult {
                    message: Some(message),
                    member_changes,
                    commit,
                    welcome,
                })
            }
            Ok(ProcessMessageResult {
                message: None,
                member_changes,
                commit,
                welcome,
            }) => {
                // This is what happens with proposals, commits, etc.
                Ok(ProcessedEventResult {
                    message: None,
                    member_changes,
                    commit,
                    welcome,
                })
            }
            Err(e) => {
                match e {
                    Error::CannotDecryptOwnMessage => {
                        tracing::debug!(target: "nostr_mls::messages::process_message", "Cannot decrypt own message, checking for cached message");
                        return Ok(ProcessedEventResult {
                            message: None,
                            member_changes: None,
                            commit: None,
                            welcome: None,
                        });
                    }
                    _ => {
                        tracing::error!(target: "nostr_mls::messages::process_message", "Error processing message: {:?}", e);
                    }
                }
                Ok(ProcessedEventResult {
                    message: None,
                    member_changes: None,
                    commit: None,
                    welcome: None,
                })
            }
        }
    }
}
