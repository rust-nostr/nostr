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

/// Default number of epochs to look back when trying to decrypt messages with older exporter secrets
const DEFAULT_EPOCH_LOOKBACK: u64 = 5;

/// MessageProcessingResult covers the full spectrum of responses that we can get back from attempting to process a message
#[derive(Debug)]
pub enum MessageProcessingResult {
    /// An application message (this is usually a message in a chat)
    ApplicationMessage(message_types::Message),
    /// Proposal message
    Proposal(UpdateGroupResult),
    /// External Join Proposal
    ExternalJoinProposal,
    /// Commit message
    Commit,
    /// Unprocessable message
    Unprocessable,
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
    fn create_message_for_event(
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
        let mut group: group_types::Group = self
            .get_group(mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupNotFound)?;

        // Create message
        let message: Vec<u8> = self.create_message_for_event(&mut mls_group, &mut rumor)?;

        // Get the rumor ID
        let rumor_id: EventId = rumor.id();

        let event = self.build_encrypted_message_event(mls_group.group_id(), message)?;

        // Create message to save to storage
        let message: message_types::Message = message_types::Message {
            id: rumor_id,
            pubkey: rumor.pubkey,
            kind: rumor.kind,
            mls_group_id: mls_group_id.clone(),
            created_at: rumor.created_at,
            content: rumor.content.clone(),
            tags: rumor.tags.clone(),
            event: rumor.clone(),
            wrapper_event_id: event.id,
            state: message_types::MessageState::Created,
        };

        // Create processed_message to track state of message
        let processed_message: message_types::ProcessedMessage = message_types::ProcessedMessage {
            wrapper_event_id: event.id,
            message_event_id: Some(rumor_id),
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::Created,
            failure_reason: None,
        };

        // Save message to storage
        self.storage()
            .save_message(message.clone())
            .map_err(|e| Error::Message(e.to_string()))?;

        // Save processed message to storage
        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;

        // Update last_message_at and last_message_id
        group.last_message_at = Some(rumor.created_at);
        group.last_message_id = Some(message.id);
        self.storage()
            .save_group(group)
            .map_err(|e| Error::Group(e.to_string()))?;

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
    /// * `Ok(ProcessedMessageContent)` - The processed message content based on message type
    /// * `Err(Error)` - If message processing fails
    fn process_message_for_group(
        &self,
        group: &mut MlsGroup,
        message_bytes: &[u8],
    ) -> Result<ProcessedMessageContent, Error> {
        let mls_message = MlsMessageIn::tls_deserialize_exact(message_bytes)?;

        tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received message: {:?}", mls_message);
        let protocol_message = mls_message.try_into_protocol_message()?;

        // Return error if group ID doesn't match
        if protocol_message.group_id() != group.group_id() {
            return Err(Error::ProtocolGroupIdMismatch);
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

        Ok(processed_message.into_content())
    }

    /// Processes an application message from a group member
    ///
    /// This internal function handles application messages (chat messages) that have been
    /// successfully decrypted. It:
    /// 1. Deserializes the message content as a Nostr event
    /// 2. Creates tracking records for the message and processing state
    /// 3. Updates the group's last message metadata
    /// 4. Stores all data in the storage provider
    ///
    /// # Arguments
    ///
    /// * `group` - The group metadata from storage
    /// * `event` - The wrapper Nostr event containing the encrypted message
    /// * `application_message` - The decrypted MLS application message
    ///
    /// # Returns
    ///
    /// * `Ok(Message)` - The processed and stored message
    /// * `Err(Error)` - If message processing or storage fails
    fn process_application_message_for_group(
        &self,
        mut group: group_types::Group,
        event: &Event,
        application_message: ApplicationMessage,
    ) -> Result<message_types::Message, Error> {
        // This is a message from a group member
        let bytes = application_message.into_bytes();
        let mut rumor: UnsignedEvent = UnsignedEvent::from_json(bytes)?;

        let rumor_id: EventId = rumor.id();

        let processed_message = message_types::ProcessedMessage {
            wrapper_event_id: event.id,
            message_event_id: Some(rumor_id),
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::Processed,
            failure_reason: None,
        };

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

        self.storage()
            .save_message(message.clone())
            .map_err(|e| Error::Message(e.to_string()))?;

        self.storage()
            .save_processed_message(processed_message.clone())
            .map_err(|e| Error::Message(e.to_string()))?;

        // Update last_message_at and last_message_id
        group.last_message_at = Some(rumor.created_at);
        group.last_message_id = Some(message.id);
        self.storage()
            .save_group(group)
            .map_err(|e| Error::Group(e.to_string()))?;

        tracing::debug!(target: "nostr_mls::messages::process_message", "Processed message: {:?}", processed_message);
        tracing::debug!(target: "nostr_mls::messages::process_message", "Message: {:?}", message);
        Ok(message)
    }

    /// Processes a proposal message from a group member
    ///
    /// This internal function handles MLS proposal messages (add/remove member proposals).
    /// Only admin members are allowed to submit proposals. The function:
    /// 1. Validates the sender is a group member and has admin privileges
    /// 2. Stores the pending proposal in the MLS group state
    /// 3. Automatically commits the proposal to the group
    /// 4. Creates a new encrypted event for the commit message
    /// 5. Updates processing state to prevent reprocessing
    ///
    /// # Arguments
    ///
    /// * `mls_group` - The MLS group to process the proposal for
    /// * `event` - The wrapper Nostr event containing the encrypted proposal
    /// * `staged_proposal` - The validated MLS proposal to process
    ///
    /// # Returns
    ///
    /// * `Ok(UpdateGroupResult)` - Contains the commit event and any welcome messages
    /// * `Err(Error)` - If proposal processing fails or sender lacks permissions
    fn process_proposal_message_for_group(
        &self,
        mls_group: &mut MlsGroup,
        event: &Event,
        staged_proposal: QueuedProposal,
    ) -> Result<UpdateGroupResult, Error> {
        match staged_proposal.sender() {
            Sender::Member(leaf_index) => {
                let member = mls_group.member_at(*leaf_index);

                match member {
                    Some(member) => {
                        // Only process proposals from admins for now
                        if self.is_member_admin(mls_group.group_id(), &member)? {
                            mls_group
                                .store_pending_proposal(self.provider.storage(), staged_proposal)
                                .map_err(|e| Error::Message(e.to_string()))?;

                            let _added_members =
                                self.pending_added_members_pubkeys(mls_group.group_id())?;

                            let mls_signer = self.load_mls_signer(mls_group)?;

                            let (commit_message, welcomes_option, _group_info) = mls_group
                                .commit_to_pending_proposals(&self.provider, &mls_signer)?;

                            let serialized_commit_message = commit_message
                                .tls_serialize_detached()
                                .map_err(|e| Error::Group(e.to_string()))?;

                            let commit_event = self.build_encrypted_message_event(
                                mls_group.group_id(),
                                serialized_commit_message,
                            )?;

                            // TODO: FUTURE Handle welcome rumors from proposals
                            // The issue is that we don't have the key_package events to get the event id to
                            // include in the welcome rumor to allow users to clean up those key packages on relays
                            let welcome_rumors: Option<Vec<UnsignedEvent>> = None;
                            if welcomes_option.is_some() {
                                return Err(Error::NotImplemented(
                                    "Processing welcome rumors from proposals is not supported"
                                        .to_string(),
                                ));
                            }

                            // Save a processed message so we don't reprocess
                            let processed_message = message_types::ProcessedMessage {
                                wrapper_event_id: event.id,
                                message_event_id: None,
                                processed_at: Timestamp::now(),
                                state: message_types::ProcessedMessageState::Processed,
                                failure_reason: None,
                            };

                            self.storage()
                                .save_processed_message(processed_message)
                                .map_err(|e| Error::Message(e.to_string()))?;

                            Ok(UpdateGroupResult {
                                evolution_event: commit_event,
                                welcome_rumors,
                            })
                        } else {
                            Err(Error::ProposalFromNonAdmin)
                        }
                    }
                    None => {
                        tracing::warn!(target: "nostr_mls::messages::process_message_for_group", "Received proposal from non-member.");
                        Err(Error::MessageFromNonMember)
                    }
                }
            }
            Sender::External(_) => {
                // TODO: FUTURE Handle external proposals from external proposal extensions
                Err(Error::NotImplemented("Processing external proposals from external proposal extensions is not supported".to_string()))
            }
            Sender::NewMemberCommit => {
                // TODO: FUTURE Handle new member from external member commits.
                Err(Error::NotImplemented(
                    "Processing external proposals for new member commits is not supported"
                        .to_string(),
                ))
            }
            Sender::NewMemberProposal => {
                // TODO: FUTURE Handle new member from external member proposals.
                Err(Error::NotImplemented(
                    "Processing external proposals for new member proposals is not supported"
                        .to_string(),
                ))
            }
        }
    }

    /// Processes a commit message from a group member
    ///
    /// This internal function handles MLS commit messages that finalize pending proposals.
    /// The function:
    /// 1. Merges the staged commit into the group state
    /// 2. Updates the group to the new epoch with new cryptographic keys
    /// 3. Saves the new exporter secret for NIP-44 encryption
    /// 4. Updates processing state to prevent reprocessing
    ///
    /// # Arguments
    ///
    /// * `mls_group` - The MLS group to merge the commit into
    /// * `event` - The wrapper Nostr event containing the encrypted commit
    /// * `staged_commit` - The validated MLS commit to merge
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If commit processing succeeds
    /// * `Err(Error)` - If commit merging or storage operations fail
    fn process_commit_message_for_group(
        &self,
        mls_group: &mut MlsGroup,
        event: &Event,
        staged_commit: StagedCommit,
    ) -> Result<(), Error> {
        mls_group
            .merge_staged_commit(&self.provider, staged_commit)
            .map_err(|e| Error::Message(e.to_string()))?;

        // Save exporter secret for the new epoch
        self.exporter_secret(mls_group.group_id())?;

        // Save a processed message so we don't reprocess
        let processed_message = message_types::ProcessedMessage {
            wrapper_event_id: event.id,
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: message_types::ProcessedMessageState::Processed,
            failure_reason: None,
        };

        self.storage()
            .save_processed_message(processed_message)
            .map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// Tries to decrypt a message using exporter secrets from multiple recent epochs
    ///
    /// This helper method attempts to decrypt a message by trying exporter secrets from
    /// the most recent epoch backwards for a configurable number of epochs. This handles
    /// the case where a message was encrypted with an older epoch's secret due to timing
    /// issues or delayed message processing.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The MLS group ID
    /// * `current_epoch` - The current epoch of the group
    /// * `encrypted_content` - The NIP-44 encrypted message content
    /// * `max_epoch_lookback` - Maximum number of epochs to search backwards (default: 5)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The decrypted message bytes
    /// * `Err(Error)` - If decryption fails with all available exporter secrets
    fn try_decrypt_with_recent_epochs(
        &self,
        group_id: &GroupId,
        current_epoch: u64,
        encrypted_content: &str,
        max_epoch_lookback: u64,
    ) -> Result<Vec<u8>, Error> {
        // Start from current epoch and go backwards
        let start_epoch = current_epoch;
        let end_epoch = current_epoch.saturating_sub(max_epoch_lookback);

        for epoch in (end_epoch..=start_epoch).rev() {
            tracing::debug!(
                target: "nostr_mls::messages::try_decrypt_with_recent_epochs",
                "Trying to decrypt with epoch {} for group {:?}",
                epoch,
                group_id
            );

            // Try to get the exporter secret for this epoch
            if let Ok(Some(secret)) = self
                .storage()
                .get_group_exporter_secret(group_id, epoch)
                .map_err(|e| Error::Group(e.to_string()))
            {
                // Convert secret to nostr keys
                if let Ok(secret_key) = SecretKey::from_slice(&secret.secret) {
                    let export_nostr_keys = Keys::new(secret_key);

                    // Try to decrypt with this epoch's secret
                    match nip44::decrypt_to_bytes(
                        export_nostr_keys.secret_key(),
                        &export_nostr_keys.public_key,
                        encrypted_content,
                    ) {
                        Ok(decrypted_bytes) => {
                            tracing::debug!(
                                target: "nostr_mls::messages::try_decrypt_with_recent_epochs",
                                "Successfully decrypted message with epoch {} for group {:?}",
                                epoch,
                                group_id
                            );
                            return Ok(decrypted_bytes);
                        }
                        Err(e) => {
                            tracing::trace!(
                                target: "nostr_mls::messages::try_decrypt_with_recent_epochs",
                                "Failed to decrypt with epoch {}: {:?}",
                                epoch,
                                e
                            );
                            // Continue to next epoch
                        }
                    }
                }
            } else {
                tracing::trace!(
                    target: "nostr_mls::messages::try_decrypt_with_recent_epochs",
                    "No exporter secret found for epoch {} in group {:?}",
                    epoch,
                    group_id
                );
            }
        }

        Err(Error::Message(format!(
            "Failed to decrypt message with any exporter secret from epochs {} to {} for group {:?}",
            end_epoch, start_epoch, group_id
        )))
    }

    /// Processes an incoming encrypted Nostr event containing an MLS message
    ///
    /// This is the main entry point for processing received messages. The function:
    /// 1. Validates the event kind (must be MlsGroupMessage)
    /// 2. Extracts and validates the group ID from event tags
    /// 3. Loads the MLS group and its cryptographic secrets
    /// 4. Decrypts the NIP-44 encrypted content using group exporter secret
    /// 5. Processes the MLS message according to its type
    /// 6. Updates message state in storage
    /// 7. Handles special cases like own messages and processing failures
    ///
    /// # Arguments
    ///
    /// * `event` - The received Nostr event containing the encrypted MLS message
    ///
    /// # Returns
    ///
    /// * `Ok(MessageProcessingResult)` - Result indicating the type of message processed
    /// * `Err(Error)` - If message processing fails
    pub fn process_message(&self, event: &Event) -> Result<MessageProcessingResult, Error> {
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

        // Load the MLS group to get the current epoch
        let mut mls_group: MlsGroup = self
            .load_mls_group(&group.mls_group_id)
            .map_err(|e| Error::Group(e.to_string()))?
            .ok_or(Error::GroupNotFound)?;

        let current_epoch = mls_group.epoch().as_u64();

        // Try to decrypt message with recent exporter secrets (fallback across epochs)
        let message_bytes: Vec<u8> = self.try_decrypt_with_recent_epochs(
            &group.mls_group_id,
            current_epoch,
            &event.content,
            DEFAULT_EPOCH_LOOKBACK,
        )?;

        match self.process_message_for_group(&mut mls_group, &message_bytes) {
            Ok(ProcessedMessageContent::ApplicationMessage(application_message)) => {
                Ok(MessageProcessingResult::ApplicationMessage(
                    self.process_application_message_for_group(group, event, application_message)?,
                ))
            }
            Ok(ProcessedMessageContent::ProposalMessage(staged_proposal)) => Ok(
                MessageProcessingResult::Proposal(self.process_proposal_message_for_group(
                    &mut mls_group,
                    event,
                    *staged_proposal,
                )?),
            ),
            Ok(ProcessedMessageContent::StagedCommitMessage(staged_commit)) => {
                self.process_commit_message_for_group(&mut mls_group, event, *staged_commit)?;
                Ok(MessageProcessingResult::Commit)
            }
            Ok(ProcessedMessageContent::ExternalJoinProposalMessage(_external_join_proposal)) => {
                // Save a processed message so we don't reprocess
                let processed_message = message_types::ProcessedMessage {
                    wrapper_event_id: event.id,
                    message_event_id: None,
                    processed_at: Timestamp::now(),
                    state: message_types::ProcessedMessageState::Processed,
                    failure_reason: None,
                };

                self.storage()
                    .save_processed_message(processed_message)
                    .map_err(|e| Error::Message(e.to_string()))?;

                Ok(MessageProcessingResult::ExternalJoinProposal)
            }
            Err(e) => {
                match e {
                    Error::CannotDecryptOwnMessage => {
                        tracing::debug!(target: "nostr_mls::messages::process_message", "Cannot decrypt own message, checking for cached message");

                        let mut processed_message = self
                            .storage()
                            .find_processed_message_by_event_id(&event.id)
                            .map_err(|e| Error::Message(e.to_string()))?
                            .ok_or(Error::Message("Processed message not found".to_string()))?;

                        // If the message is created, we need to update the state of the message and processed message
                        // If it's already processed, we don't need to do anything
                        match processed_message.state {
                            message_types::ProcessedMessageState::Created => {
                                let message_event_id: EventId =
                                    processed_message.message_event_id.ok_or(Error::Message(
                                        "Message event ID not found".to_string(),
                                    ))?;

                                let mut message = self
                                    .get_message(&message_event_id)?
                                    .ok_or(Error::Message("Message not found".to_string()))?;

                                message.state = message_types::MessageState::Processed;
                                self.storage()
                                    .save_message(message)
                                    .map_err(|e| Error::Message(e.to_string()))?;

                                processed_message.state =
                                    message_types::ProcessedMessageState::Processed;
                                self.storage()
                                    .save_processed_message(processed_message.clone())
                                    .map_err(|e| Error::Message(e.to_string()))?;

                                tracing::debug!(target: "nostr_mls::messages::process_message", "Updated state of own cached message");
                                let message = self
                                    .get_message(&message_event_id)?
                                    .ok_or(Error::MessageNotFound)?;
                                Ok(MessageProcessingResult::ApplicationMessage(message))
                            }
                            message_types::ProcessedMessageState::ProcessedCommit => {
                                tracing::debug!(target: "nostr_mls::messages::process_message", "Message already processed as a commit");
                                Ok(MessageProcessingResult::Commit)
                            }
                            message_types::ProcessedMessageState::Processed
                            | message_types::ProcessedMessageState::Failed => {
                                tracing::debug!(target: "nostr_mls::messages::process_message", "Message cannot be processed (already processed or failed)");
                                Ok(MessageProcessingResult::Unprocessable)
                            }
                        }
                    }
                    _ => {
                        tracing::error!(target: "nostr_mls::messages::process_message", "Error processing message: {:?}", e);
                        let processed_message = message_types::ProcessedMessage {
                            wrapper_event_id: event.id,
                            message_event_id: None,
                            processed_at: Timestamp::now(),
                            state: message_types::ProcessedMessageState::Failed,
                            failure_reason: Some(e.to_string()),
                        };
                        self.storage()
                            .save_processed_message(processed_message)
                            .map_err(|e| Error::Message(e.to_string()))?;

                        Ok(MessageProcessingResult::Unprocessable)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nostr::{Event, EventBuilder, Keys, Kind, PublicKey, RelayUrl, Tag, TagKind};
    use nostr_mls_memory_storage::NostrMlsMemoryStorage;

    use super::*;
    use crate::tests::create_test_nostr_mls;

    /// Helper function to create test group members
    fn create_test_group_members() -> (Keys, Vec<Keys>, Vec<PublicKey>) {
        let creator = Keys::generate();
        let member1 = Keys::generate();
        let member2 = Keys::generate();

        let creator_pk = creator.public_key();
        let members = vec![member1, member2];
        let admins = vec![creator_pk, members[0].public_key()];

        (creator, members, admins)
    }

    /// Helper function to create a key package event
    fn create_key_package_event(
        nostr_mls: &crate::NostrMls<NostrMlsMemoryStorage>,
        member_keys: &Keys,
    ) -> Event {
        let relays = vec![RelayUrl::parse("wss://test.relay").unwrap()];
        let (key_package_hex, tags) = nostr_mls
            .create_key_package_for_event(&member_keys.public_key(), relays)
            .expect("Failed to create key package");

        EventBuilder::new(Kind::MlsKeyPackage, key_package_hex)
            .tags(tags.to_vec())
            .sign_with_keys(member_keys)
            .expect("Failed to sign event")
    }

    /// Helper function to create a test group and return the group ID
    fn create_test_group(
        nostr_mls: &crate::NostrMls<NostrMlsMemoryStorage>,
        creator: &Keys,
        members: &[Keys],
        admins: &[PublicKey],
    ) -> GroupId {
        let creator_pk = creator.public_key();

        // Create key package events for initial members
        let mut initial_key_package_events = Vec::new();
        for member_keys in members {
            let key_package_event = create_key_package_event(nostr_mls, member_keys);
            initial_key_package_events.push(key_package_event);
        }

        // Create the group
        let create_result = nostr_mls
            .create_group(
                "Test Group",
                "A test group for message testing",
                Option::<String>::None,
                Option::<SecretKey>::None,
                &creator_pk,
                initial_key_package_events,
                admins.to_vec(),
                vec![RelayUrl::parse("wss://test.relay").unwrap()],
            )
            .expect("Failed to create group");

        let group_id = create_result.group.mls_group_id.clone();

        // Merge the pending commit to apply the member additions
        nostr_mls
            .merge_pending_commit(&group_id)
            .expect("Failed to merge pending commit");

        group_id
    }

    /// Helper function to create a test message rumor
    fn create_test_rumor(sender_keys: &Keys, content: &str) -> UnsignedEvent {
        EventBuilder::new(Kind::TextNote, content).build(sender_keys.public_key())
    }

    #[test]
    fn test_get_message_not_found() {
        let nostr_mls = create_test_nostr_mls();
        let non_existent_event_id = EventId::all_zeros();

        let result = nostr_mls.get_message(&non_existent_event_id);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_messages_empty_group() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        let messages = nostr_mls
            .get_messages(&group_id)
            .expect("Failed to get messages");
        assert!(messages.is_empty());
    }

    #[test]
    fn test_create_message_success() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        // Create a test message
        let mut rumor = create_test_rumor(&creator, "Hello, world!");
        let rumor_id = rumor.id();

        let result = nostr_mls.create_message(&group_id, rumor);
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.kind, Kind::MlsGroupMessage);

        // Verify the message was stored
        let stored_message = nostr_mls
            .get_message(&rumor_id)
            .expect("Failed to get message")
            .expect("Message should exist");

        assert_eq!(stored_message.id, rumor_id);
        assert_eq!(stored_message.content, "Hello, world!");
        assert_eq!(stored_message.state, message_types::MessageState::Created);
        assert_eq!(stored_message.wrapper_event_id, event.id);
    }

    #[test]
    fn test_create_message_group_not_found() {
        let nostr_mls = create_test_nostr_mls();
        let creator = Keys::generate();
        let rumor = create_test_rumor(&creator, "Hello, world!");
        let non_existent_group_id = GroupId::from_slice(&[1, 2, 3, 4]);

        let result = nostr_mls.create_message(&non_existent_group_id, rumor);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::GroupNotFound));
    }

    #[test]
    fn test_create_message_updates_group_metadata() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        // Get initial group state
        let initial_group = nostr_mls
            .get_group(&group_id)
            .expect("Failed to get group")
            .expect("Group should exist");
        assert!(initial_group.last_message_at.is_none());
        assert!(initial_group.last_message_id.is_none());

        // Create a message
        let mut rumor = create_test_rumor(&creator, "Hello, world!");
        let rumor_id = rumor.id();
        let rumor_timestamp = rumor.created_at;

        let _event = nostr_mls
            .create_message(&group_id, rumor)
            .expect("Failed to create message");

        // Verify group metadata was updated
        let updated_group = nostr_mls
            .get_group(&group_id)
            .expect("Failed to get group")
            .expect("Group should exist");

        assert_eq!(updated_group.last_message_at, Some(rumor_timestamp));
        assert_eq!(updated_group.last_message_id, Some(rumor_id));
    }

    #[test]
    fn test_process_message_invalid_kind() {
        let nostr_mls = create_test_nostr_mls();
        let creator = Keys::generate();

        // Create an event with wrong kind
        let event = EventBuilder::new(Kind::TextNote, "test content")
            .sign_with_keys(&creator)
            .expect("Failed to sign event");

        let result = nostr_mls.process_message(&event);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnexpectedEvent { .. }));
    }

    #[test]
    fn test_process_message_missing_group_id_tag() {
        let nostr_mls = create_test_nostr_mls();
        let creator = Keys::generate();

        // Create an event without group ID tag
        let event = EventBuilder::new(Kind::MlsGroupMessage, "test content")
            .sign_with_keys(&creator)
            .expect("Failed to sign event");

        let result = nostr_mls.process_message(&event);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Message(_)));
    }

    #[test]
    fn test_process_message_group_not_found() {
        let nostr_mls = create_test_nostr_mls();
        let creator = Keys::generate();

        // Create a valid MLS group message event with non-existent group ID
        let fake_group_id = hex::encode([1u8; 32]);
        let tag = Tag::custom(TagKind::h(), [fake_group_id]);

        let event = EventBuilder::new(Kind::MlsGroupMessage, "encrypted_content")
            .tag(tag)
            .sign_with_keys(&creator)
            .expect("Failed to sign event");

        let result = nostr_mls.process_message(&event);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::GroupNotFound));
    }

    #[test]
    fn test_message_state_tracking() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        // Create a message
        let mut rumor = create_test_rumor(&creator, "Test message state");
        let rumor_id = rumor.id();

        let event = nostr_mls
            .create_message(&group_id, rumor)
            .expect("Failed to create message");

        // Verify initial state
        let message = nostr_mls
            .get_message(&rumor_id)
            .expect("Failed to get message")
            .expect("Message should exist");

        assert_eq!(message.state, message_types::MessageState::Created);

        // Verify processed message state
        let processed_message = nostr_mls
            .storage()
            .find_processed_message_by_event_id(&event.id)
            .expect("Failed to get processed message")
            .expect("Processed message should exist");

        assert_eq!(
            processed_message.state,
            message_types::ProcessedMessageState::Created
        );
        assert_eq!(processed_message.message_event_id, Some(rumor_id));
        assert_eq!(processed_message.wrapper_event_id, event.id);
    }

    #[test]
    fn test_get_messages_for_group() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        // Create multiple messages
        let rumor1 = create_test_rumor(&creator, "First message");
        let rumor2 = create_test_rumor(&creator, "Second message");

        let _event1 = nostr_mls
            .create_message(&group_id, rumor1)
            .expect("Failed to create first message");
        let _event2 = nostr_mls
            .create_message(&group_id, rumor2)
            .expect("Failed to create second message");

        // Get all messages for the group
        let messages = nostr_mls
            .get_messages(&group_id)
            .expect("Failed to get messages");

        assert_eq!(messages.len(), 2);

        // Verify message contents
        let contents: Vec<&str> = messages.iter().map(|m| m.content.as_str()).collect();
        assert!(contents.contains(&"First message"));
        assert!(contents.contains(&"Second message"));

        // Verify all messages belong to the correct group
        for message in &messages {
            assert_eq!(message.mls_group_id, group_id);
        }
    }

    #[test]
    fn test_message_processing_result_variants() {
        // Test that MessageProcessingResult variants can be created and matched
        let dummy_message = message_types::Message {
            id: EventId::all_zeros(),
            pubkey: PublicKey::from_hex(
                "8a9de562cbbed225b6ea0118dd3997a02df92c0bffd2224f71081a7450c3e549",
            )
            .unwrap(),
            kind: Kind::TextNote,
            mls_group_id: GroupId::from_slice(&[1, 2, 3, 4]),
            created_at: Timestamp::now(),
            content: "Test".to_string(),
            tags: Tags::new(),
            event: EventBuilder::new(Kind::TextNote, "Test").build(
                PublicKey::from_hex(
                    "8a9de562cbbed225b6ea0118dd3997a02df92c0bffd2224f71081a7450c3e549",
                )
                .unwrap(),
            ),
            wrapper_event_id: EventId::all_zeros(),
            state: message_types::MessageState::Processed,
        };

        let app_result = MessageProcessingResult::ApplicationMessage(dummy_message);
        let commit_result = MessageProcessingResult::Commit;
        let external_join_result = MessageProcessingResult::ExternalJoinProposal;
        let unprocessable_result = MessageProcessingResult::Unprocessable;

        // Test that we can match on variants
        match app_result {
            MessageProcessingResult::ApplicationMessage(_) => {}
            _ => panic!("Expected ApplicationMessage variant"),
        }

        match commit_result {
            MessageProcessingResult::Commit => {}
            _ => panic!("Expected Commit variant"),
        }

        match external_join_result {
            MessageProcessingResult::ExternalJoinProposal => {}
            _ => panic!("Expected ExternalJoinProposal variant"),
        }

        match unprocessable_result {
            MessageProcessingResult::Unprocessable => {}
            _ => panic!("Expected Unprocessable variant"),
        }
    }

    #[test]
    fn test_message_content_preservation() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        // Test with various content types
        let test_cases = vec![
            "Simple text message",
            "Message with emojis 🚀 🎉 ✨",
            "Message with\nmultiple\nlines",
            "Message with special chars: !@#$%^&*()",
            "Minimal content",
        ];

        for content in test_cases {
            let mut rumor = create_test_rumor(&creator, content);
            let rumor_id = rumor.id();

            let _event = nostr_mls
                .create_message(&group_id, rumor)
                .expect("Failed to create message");

            let stored_message = nostr_mls
                .get_message(&rumor_id)
                .expect("Failed to get message")
                .expect("Message should exist");

            assert_eq!(stored_message.content, content);
            assert_eq!(stored_message.pubkey, creator.public_key());
        }
    }

    #[test]
    fn test_create_message_ensures_rumor_id() {
        let nostr_mls = create_test_nostr_mls();
        let (creator, members, admins) = create_test_group_members();
        let group_id = create_test_group(&nostr_mls, &creator, &members, &admins);

        // Create a rumor - EventBuilder.build() ensures the ID is set
        let rumor = create_test_rumor(&creator, "Test message");

        let result = nostr_mls.create_message(&group_id, rumor);
        assert!(result.is_ok());

        // The message should have been stored with a valid ID
        let event = result.unwrap();
        let messages = nostr_mls
            .get_messages(&group_id)
            .expect("Failed to get messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].wrapper_event_id, event.id);
    }
}
