// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP117: The Double Ratchet Algorithm
//!
//! <https://github.com/nostr-protocol/nips/blob/master/117.md>

use core::fmt;
use std::collections::HashMap;

use hashes::sha256::Hash as Sha256Hash;
use hashes::{Hash, Hmac};

use super::nip44::{self, v2, ConversationKey, Version};
use crate::event::builder;
use crate::key::public_key::PublicKey;
use crate::util::hkdf;
use crate::{Event, EventBuilder, JsonUtil, Keys, Kind, Tag, TagKind, TagStandard, UnsignedEvent};

const KEY_DERIVATION_HKDF_EXPAND_SIZE: usize = 32;
const MAX_SKIP: usize = 1000;

/// Double Ratchet error
#[derive(Debug)]
pub enum Error {
    /// NIP44 error
    NIP44(nip44::Error),
    /// Event Builder error
    EventBuilder(builder::Error),
    /// JSON error
    Json(serde_json::Error),
    /// The current keypair is missing
    CurrentKeysMissing,
    /// The sending chain key is missing
    SendingChainKeyMissing,
    /// The receiving chain key is missing
    ReceivingChainKeyMissing,
    /// The header is missing in the event tags
    HeaderMissing,
    /// Failed to decrypt header with current and skipped header keys
    HeaderDecryptionFailed,
    /// Too many skipped messages
    TooManySkippedMessages,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP44(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::CurrentKeysMissing => write!(f, "current keypair is missing"),
            Self::SendingChainKeyMissing => write!(f, "sending chain key is missing"),
            Self::ReceivingChainKeyMissing => write!(f, "receiving chain key is missing"),
            Self::HeaderMissing => write!(f, "header is missing in the event tags"),
            Self::HeaderDecryptionFailed => write!(
                f,
                "failed to decrypt header with current and skipped header keys"
            ),
            Self::TooManySkippedMessages => write!(f, "too many skipped messages"),
        }
    }
}

impl From<nip44::Error> for Error {
    fn from(e: nip44::Error) -> Self {
        Self::NIP44(e)
    }
}

impl From<builder::Error> for Error {
    fn from(e: builder::Error) -> Self {
        Self::EventBuilder(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Double Ratchet Header
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DoubleRatchetHeader {
    /// Current message number
    pub number: usize,
    /// Next public key
    pub next_public_key: PublicKey,
    /// Previous chain length
    pub previous_chain_length: usize,
}

impl JsonUtil for DoubleRatchetHeader {
    type Err = serde_json::Error;
}

/// Cache of skipped keys for handling out-of-order messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedKeys {
    /// Header keys
    pub header_keys: Vec<ConversationKey>,
    /// Message keys
    pub message_keys: HashMap<usize, ConversationKey>,
}

/// Double Ratchet Session State
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleRatchetSessionState {
    /// Root key used to derive new sending/receiving chain keys
    pub root_key: Vec<u8>,
    /// The other party's current Nostr public key
    pub their_current_public_key: Option<PublicKey>,
    /// The other party's next Nostr public key
    pub their_next_public_key: PublicKey,
    /// Our current keypair used for this session
    pub our_current_key: Option<Keys>,
    /// Our next keypair, used when ratcheting forward. It is advertised in messages we send.
    pub our_next_key: Keys,
    /// Key for decrypting incoming messages in the current chain
    pub receiving_chain_key: Option<Vec<u8>>,
    /// Key for encrypting outgoing messages in the current chain
    pub sending_chain_key: Option<Vec<u8>>,
    /// Number of messages sent in the current sending chain
    pub sending_chain_message_number: usize,
    /// Number of messages received in the current receiving chain
    pub receiving_chain_message_number: usize,
    /// Number of messages sent in the previous sending chain
    pub previous_sending_chain_message_count: usize,
    /// Cache of message & header keys for handling out-of-order messages
    pub skipped_keys: HashMap<PublicKey, SkippedKeys>,
}

impl DoubleRatchetSessionState {
    /// Generate new session state as chat initiator
    pub fn new_initiator(
        their_ephemeral_public_key: PublicKey,
        our_ephemeral_key: &Keys,
        shared_secret: &[u8],
    ) -> Result<Self, Error> {
        let our_next_key: Keys = Keys::generate();

        // Derive NIP44-v2 conversation key
        let conversation_key: v2::ConversationKey = v2::ConversationKey::derive(
            our_ephemeral_key.secret_key(),
            &their_ephemeral_public_key,
        )?;

        // Use ephemeral ECDH to derive rootKey and sendingChainKey
        let keys: Vec<Vec<u8>> = kdf(shared_secret, conversation_key.as_bytes(), 2);
        let mut keys_iterator = keys.into_iter();

        let root_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");
        let sending_chain_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");

        Ok(Self {
            root_key,
            their_current_public_key: None,
            their_next_public_key: their_ephemeral_public_key,
            our_current_key: None,
            our_next_key,
            receiving_chain_key: None,
            sending_chain_key: Some(sending_chain_key),
            sending_chain_message_number: 0,
            receiving_chain_message_number: 0,
            previous_sending_chain_message_count: 0,
            skipped_keys: HashMap::new(),
        })
    }

    /// Generate new session state as chat receiver
    pub fn new_receiver(
        their_ephemeral_public_key: PublicKey,
        our_ephemeral_key: Keys,
        shared_secret: Vec<u8>,
    ) -> Self {
        Self {
            root_key: shared_secret,
            their_current_public_key: None,
            their_next_public_key: their_ephemeral_public_key,
            our_current_key: None,
            our_next_key: our_ephemeral_key,
            receiving_chain_key: None,
            sending_chain_key: None,
            sending_chain_message_number: 0,
            receiving_chain_message_number: 0,
            previous_sending_chain_message_count: 0,
            skipped_keys: HashMap::new(),
        }
    }

    /// Encrypts data using the Double Ratchet mechanism
    fn ratchet_encrypt<T>(&mut self, data: T) -> Result<(DoubleRatchetHeader, String), Error>
    where
        T: AsRef<[u8]>,
    {
        let sending_chain_key: &[u8] = self
            .sending_chain_key
            .as_ref()
            .ok_or(Error::SendingChainKeyMissing)?;

        // Derive the new sending chain key and message key using the KDF.
        let keys: Vec<Vec<u8>> = kdf(sending_chain_key, &[1u8], 2);
        let mut keys_iterator = keys.into_iter();

        let new_sending_chain_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");
        let message_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");

        // Convert message key to NIP44-v2 conversation key
        let conversation_key: ConversationKey =
            ConversationKey::V2(v2::ConversationKey::from_slice(&message_key)?);

        // Update the sending chain key in the session state.
        self.sending_chain_key = Some(new_sending_chain_key);

        // Create the header for the message.
        let header = DoubleRatchetHeader {
            number: self.sending_chain_message_number,
            next_public_key: self.our_next_key.public_key,
            previous_chain_length: self.previous_sending_chain_message_count,
        };

        // Increment the sending chain message counter.
        self.sending_chain_message_number += 1;

        // Encrypt the plaintext with the derived message key.
        let encrypted_message: String =
            nip44::encrypt_with_conversation_key(&conversation_key, data)?;

        // Return the header and the encrypted message as a tuple.
        Ok((header, encrypted_message))
    }

    /// Create a new message
    pub fn create_message(&mut self, mut rumor: UnsignedEvent) -> Result<Event, Error> {
        // Ensure the rumor has the ID
        rumor.ensure_id();

        // Serialize rumor to JSON
        let rumor_json: String = rumor.as_json();

        // Ratchet_encryption
        let (header, encrypted_message) = self.ratchet_encrypt(&rumor_json)?;

        // Generate conversation key and encrypt header
        let our_current_key: &Keys = self
            .our_current_key
            .as_ref()
            .ok_or(Error::CurrentKeysMissing)?;
        let conversation_key: ConversationKey = ConversationKey::derive(
            our_current_key.secret_key(),
            &self.their_next_public_key,
            Version::V2,
        )?;
        let encrypted_header: String =
            nip44::encrypt_with_conversation_key(&conversation_key, header.as_json())?;

        // Build event
        Ok(
            EventBuilder::new(Kind::DoubleRatchetMessage, encrypted_message)
                .tag(Tag::header(encrypted_header))
                .sign_with_keys(our_current_key)?,
        )
    }

    fn decrypt_header(&self, event: &Event) -> Result<(DoubleRatchetHeader, bool, bool), Error> {
        // Get encrypted header
        let encrypted_header: &str = match event.tags.find_standardized(TagKind::Header) {
            Some(TagStandard::Header(encrypted_header)) => encrypted_header,
            Some(..) | None => return Err(Error::HeaderMissing),
        };

        if let Some(our_current_key) = &self.our_current_key {
            let current_secret: ConversationKey =
                ConversationKey::derive(our_current_key.secret_key(), &event.pubkey, Version::V2)?;

            // Don't propagate error here.
            // If the decryption fails, try with our current key at next step.
            if let Ok(header_json) =
                nip44::decrypt_with_conversation_key(&current_secret, encrypted_header)
            {
                let header: DoubleRatchetHeader = DoubleRatchetHeader::from_json(header_json)?;
                return Ok((header, false, false));
            }

            // Decryption failed, try with our next key
        }

        let next_secret: ConversationKey =
            ConversationKey::derive(self.our_next_key.secret_key(), &event.pubkey, Version::V2)?;

        // Don't propagate error here.
        // If the decryption fails, try with skipped keys at next step.
        if let Ok(header_json) =
            nip44::decrypt_with_conversation_key(&next_secret, encrypted_header)
        {
            let header: DoubleRatchetHeader = DoubleRatchetHeader::from_json(header_json)?;
            return Ok((header, true, false));
        }

        if let Some(skipped_key) = self.skipped_keys.get(&event.pubkey) {
            for key in skipped_key.header_keys.iter() {
                // Don't propagate error here.
                // If the decryption fails, try with next skipped key at next iteration.
                if let Ok(header_json) =
                    nip44::decrypt_with_conversation_key(&key, encrypted_header)
                {
                    let header: DoubleRatchetHeader = DoubleRatchetHeader::from_json(header_json)?;
                    return Ok((header, false, true));
                }

                // Decryption failed, try with next skipped key
            }
        }

        Err(Error::HeaderDecryptionFailed)
    }

    fn skip_message_keys(&mut self, until: usize, nostr_sender: PublicKey) -> Result<(), Error> {
        if until <= self.receiving_chain_message_number {
            return Ok(());
        }

        if until > self.receiving_chain_message_number + MAX_SKIP {
            return Err(Error::TooManySkippedMessages);
        }

        match self.skipped_keys.get_mut(&nostr_sender) {
            Some(skipped_keys) => {
                while self.receiving_chain_message_number < until {
                    if let Some(receiving_chain_key) = &self.receiving_chain_key {
                        let keys = kdf(receiving_chain_key, &[1], 2);
                        let mut keys_iterator = keys.into_iter();

                        let new_receiving_chain_key: Vec<u8> =
                            keys_iterator.next().expect("Expected 2 outputs");
                        let message_key: Vec<u8> =
                            keys_iterator.next().expect("Expected 2 outputs");
                        let conversation_key: ConversationKey =
                            ConversationKey::V2(v2::ConversationKey::from_slice(&message_key)?);

                        self.receiving_chain_key = Some(new_receiving_chain_key);

                        skipped_keys
                            .message_keys
                            .insert(self.receiving_chain_message_number, conversation_key);

                        self.receiving_chain_message_number += 1;
                    }

                    // TODO: should return error if receiving_chain_key is None?
                }
            }
            None => {
                let mut header_keys: Vec<ConversationKey> = Vec::new();

                if let Some(our_current_key) = &self.our_current_key {
                    let current_secret: ConversationKey = ConversationKey::derive(
                        our_current_key.secret_key(),
                        &nostr_sender,
                        Version::V2,
                    )?;
                    header_keys.push(current_secret);
                }

                let next_secret: ConversationKey = ConversationKey::derive(
                    self.our_next_key.secret_key(),
                    &nostr_sender,
                    Version::V2,
                )?;
                header_keys.push(next_secret);
            }
        }

        Ok(())
    }

    fn ratchet_step(&mut self) -> Result<(), Error> {
        self.previous_sending_chain_message_count = self.sending_chain_message_number;
        self.sending_chain_message_number = 0;
        self.receiving_chain_message_number = 0;

        let conversation_key: ConversationKey = ConversationKey::derive(
            self.our_next_key.secret_key(),
            &self.their_next_public_key,
            Version::V2,
        )?;
        let keys = kdf(&self.root_key, conversation_key.as_bytes(), 2);
        let mut keys_iterator = keys.into_iter();

        let their_root_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");
        let receiving_chain_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");

        self.receiving_chain_key = Some(receiving_chain_key);

        // Update our keypair
        self.our_current_key = Some(self.our_next_key.clone());
        self.our_next_key = Keys::generate();

        let conversation_key: ConversationKey = ConversationKey::derive(
            self.our_next_key.secret_key(),
            &self.their_next_public_key,
            Version::V2,
        )?;
        let keys = kdf(&their_root_key, conversation_key.as_bytes(), 2);
        let mut keys_iterator = keys.into_iter();

        let root_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");
        let sending_chain_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");

        self.root_key = root_key;
        self.sending_chain_key = Some(sending_chain_key);

        Ok(())
    }

    fn try_skipped_message_keys(
        &mut self,
        header: DoubleRatchetHeader,
        ciphertext: &str,
        nostr_sender: PublicKey,
    ) -> Result<Option<String>, Error> {
        match self.skipped_keys.get_mut(&nostr_sender) {
            Some(skipped_keys) => {
                match skipped_keys.message_keys.remove(&header.number) {
                    Some(conversation_key) => Ok(Some(nip44::decrypt_with_conversation_key(
                        &conversation_key,
                        ciphertext,
                    )?)),
                    None => {
                        // No message key found
                        Ok(None)
                    }
                }
            }
            None => {
                // No skipped keys for this sender
                Ok(None)
            }
        }
    }

    fn ratchet_decrypt(
        &mut self,
        header: DoubleRatchetHeader,
        ciphertext: &str,
        nostr_sender: PublicKey,
    ) -> Result<String, Error> {
        if let Some(plaintext) = self.try_skipped_message_keys(header, ciphertext, nostr_sender)? {
            return Ok(plaintext);
        }

        self.skip_message_keys(header.number, nostr_sender)?;

        let receiving_chain_key: &[u8] = self
            .receiving_chain_key
            .as_ref()
            .ok_or(Error::ReceivingChainKeyMissing)?;
        let keys = kdf(receiving_chain_key, &[1], 2);
        let mut keys_iterator = keys.into_iter();

        let new_receiving_chain_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");
        let message_key: Vec<u8> = keys_iterator.next().expect("Expected 2 outputs");

        let conversation_key: ConversationKey =
            ConversationKey::V2(v2::ConversationKey::from_slice(&message_key)?);

        self.receiving_chain_key = Some(new_receiving_chain_key);
        self.receiving_chain_message_number += 1;

        Ok(nip44::decrypt_with_conversation_key(
            &conversation_key,
            ciphertext,
        )?)
    }

    /// Handle received event
    ///
    /// Returns [`Ok(None)`] if the event was already processed
    pub fn handle_received_event(&mut self, event: &Event) -> Result<Option<UnsignedEvent>, Error> {
        let (header, should_ratchet, is_skipped) = self.decrypt_header(event)?;

        if !is_skipped {
            if self.their_next_public_key != header.next_public_key {
                self.their_current_public_key = Some(self.their_next_public_key);
                self.their_next_public_key = header.next_public_key;

                // TODO: the client have to create an auto-closing subscription for {"authors":["<their_next_public_key>"],"kinds":[1060]}
            }

            if should_ratchet {
                self.skip_message_keys(header.previous_chain_length, event.pubkey)?;
                self.ratchet_step()?;
            }
        } else {
            if let Some(skipped_keys) = self.skipped_keys.get(&event.pubkey) {
                // Check if the header number doesn't exist in message keys
                if !skipped_keys.message_keys.contains_key(&header.number) {
                    // Maybe we already processed this message — no error
                    return Ok(None);
                }
            }
        }

        let unsigned_event_json: String =
            self.ratchet_decrypt(header, &event.content, event.pubkey)?;
        Ok(Some(serde_json::from_str(&unsigned_event_json)?))
    }
}

/// Double Ratchet keys derivation
pub fn kdf(input1: &[u8], input2: &[u8], num_outputs: u8) -> Vec<Vec<u8>> {
    let prk: Hmac<Sha256Hash> = hkdf::extract(input1, input2);

    let mut outputs: Vec<Vec<u8>> = Vec::with_capacity(num_outputs as usize);

    for i in 0..num_outputs {
        outputs.push(hkdf::expand(
            prk.as_byte_array(),
            &[i],
            KEY_DERIVATION_HKDF_EXPAND_SIZE,
        ));
    }

    outputs
}
