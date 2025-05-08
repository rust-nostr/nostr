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
    /// * `Ok(Some(UnsignedEvent))` - For application messages, the decrypted event
    /// * `Ok(None)` - For protocol messages (proposals, commits)
    /// * `Err(Error)` - If message processing fails
    fn process_message_for_group(
        &self,
        group: &mut MlsGroup,
        message_bytes: &[u8],
    ) -> Result<Option<UnsignedEvent>, Error> {
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
                Ok(Some(rumor))
            }
            ProcessedMessageContent::ProposalMessage(staged_proposal) => {
                // This is a proposal message
                tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received proposal message: {:?}", staged_proposal);
                // TODO: Handle proposal message
                Ok(None)
            }
            ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
                // This is a commit message
                tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received commit message: {:?}", staged_commit);
                // TODO: Handle commit message
                Ok(None)
            }
            ProcessedMessageContent::ExternalJoinProposalMessage(external_join_proposal) => {
                tracing::debug!(target: "nostr_mls::messages::process_message_for_group", "Received external join proposal message: {:?}", external_join_proposal);
                // TODO: Handle external join proposal
                Ok(None)
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
    /// * `mls_group_id` - The MLS group ID
    /// * `event` - The received Nostr event
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If message processing succeeds
    /// * `Err(Error)` - If message processing fails
    pub fn process_message(&self, event: &Event) -> Result<Option<message_types::Message>, Error> {
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

        let mut group = self
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
            Ok(Some(mut rumor)) => {
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
                Ok(Some(message))
            }
            Ok(None) => {
                // This is what happens with proposals, commits, etc.
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

                Ok(None)
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
                        let message = self.get_message(&message_event_id)?;
                        return Ok(message);
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
                    }
                }
                Ok(None)
            }
        }
    }
}
