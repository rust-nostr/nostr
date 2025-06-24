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
use openmls::prelude::BasicCredential;
use openmls_basic_credential::SignatureKeyPair;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};

use crate::error::Error;
use crate::prelude::*;
use crate::NostrMls;

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
                        // Only process commits from admins for now
                        if self.is_member_admin(mls_group.group_id(), &member)? {
                            mls_group
                                .store_pending_proposal(self.provider.storage(), staged_proposal)
                                .map_err(|e| Error::Message(e.to_string()))?;

                            let mls_signer = self.load_mls_signer(&mls_group)?;

                            let (commit_message, welcomes_option, _group_info) = mls_group
                                .commit_to_pending_proposals(&self.provider, &mls_signer)?;

                            let serialized_commit_message = commit_message
                                .tls_serialize_detached()
                                .map_err(|e| Error::Group(e.to_string()))?;

                            let commit_event = self.build_encrypted_message_event(
                                mls_group.group_id(),
                                serialized_commit_message,
                            )?;

                            // TODO: welcomes
                            // We need to get the list of added users covered by the proposal.
                            // Then create unsigned rumors for each.

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
                                welcome_rumors: Some(Vec::new()),
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

        /// Extracts public keys of newly added members from a staged commit
    ///
    /// This helper method examines the proposals within a staged commit to identify
    /// any Add proposals that would add new members to the group. For each new member,
    /// it extracts their public key from their LeafNode.
    ///
    /// # Arguments
    ///
    /// * `staged_commit` - The staged commit to examine for new members
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PublicKey>)` - List of public keys for newly added members
    /// * `Err(Error)` - If there's an error extracting member information
    pub(crate) fn extract_added_members_pubkeys(
        &self,
        staged_commit: &StagedCommit,
    ) -> Result<Vec<PublicKey>, Error> {
        let mut added_pubkeys = Vec::new();

        // Get the queued proposals from the staged commit
        for proposal in staged_commit.queued_proposals() {
            if let Proposal::Add(add_proposal) = proposal.proposal() {
                // Extract the public key from the LeafNode using the same pattern as groups.rs
                let leaf_node = add_proposal.key_package().leaf_node();
                let pubkey = self.pubkey_for_leaf_node(leaf_node)?;
                added_pubkeys.push(pubkey);
            }
        }

        Ok(added_pubkeys)
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

                        let message_event_id: EventId = processed_message
                            .message_event_id
                            .ok_or(Error::Message("Message event ID not found".to_string()))?;

                        // If the message is created, we need to update the state of the message and processed message
                        // If it's already processed, we don't need to do anything
                        match processed_message.state {
                            message_types::ProcessedMessageState::Created => {
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
                            }
                            message_types::ProcessedMessageState::Processed => {
                                tracing::debug!(target: "nostr_mls::messages::process_message", "Message already processed");
                            }
                            message_types::ProcessedMessageState::Failed => {
                                tracing::debug!(target: "nostr_mls::messages::process_message", "Message previously failed to process");
                            }
                        }
                        let message = self
                            .get_message(&message_event_id)?
                            .ok_or(Error::MessageNotFound)?;
                        Ok(MessageProcessingResult::ApplicationMessage(message))
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
