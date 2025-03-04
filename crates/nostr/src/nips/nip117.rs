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
use crate::{Event, EventBuilder, JsonUtil, Keys, Kind, Tag, UnsignedEvent};

const KEY_DERIVATION_HKDF_EXPAND_SIZE: usize = 32;

/// Double Ratchet error
#[derive(Debug)]
pub enum Error {
    /// NIP44 error
    NIP44(nip44::Error),
    /// Event Builder error
    EventBuilder(builder::Error),
    /// The current keypair is missing
    CurrentKeysMissing,
    /// The sending chain key is missing
    SendingChainKeyMissing,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP44(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::CurrentKeysMissing => write!(f, "current keypair is missing"),
            Self::SendingChainKeyMissing => write!(f, "sending chain key is missing"),
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
    pub header_keys: Vec<Vec<u8>>,
    /// Message keys
    pub message_keys: HashMap<u64, Vec<u8>>,
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
    pub skipped_keys: HashMap<String, SkippedKeys>,
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
