// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP17: Private Direct Message
//!
//! <https://github.com/nostr-protocol/nips/blob/master/17.md>

#![allow(clippy::wrong_self_convention)]

use url::Url;

use crate::event::builder::{Error, EventBuilder};
use crate::{Event, Kind, RelayUrl, Tag, TagStandard};

/// Encrypted file
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncryptedFile {
    /// URL of the encrypted file
    pub url: Url,
    /// File type (e.g. "image/png", "application/pdf")
    pub file_type: String,
    /// Decryption key
    pub decryption_key: String,
    /// Decryption nonce
    pub decryption_nonce: u128,
    /// SHA256 hash of the file
    pub hash: hashes::sha256::Hash,
}

impl EncryptedFile {
    pub(crate) fn to_event_builder(self) -> Result<EventBuilder, Error> {
        let mut tags: Vec<Tag> = Vec::with_capacity(5);

        // Add file type
        if !self.file_type.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::FileType(
                self.file_type,
            )));
        }

        // Add decryption key
        if !self.decryption_key.is_empty() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::DecryptionKey(self.decryption_key),
            ));
        }

        // Add decryption nonce
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::DecryptionNonce(self.decryption_nonce),
        ));

        // Add hash
        tags.push(Tag::from_standardized_without_cell(TagStandard::Sha256(
            self.hash,
        )));

        // Add encryption algorithm
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::EncryptionAlgorithm,
        ));

        // Build
        Ok(EventBuilder::new(Kind::FileMessage, self.url).tags(tags))
    }
}

/// Extracts the relay list
///
/// This function doesn't verify if the event kind is [`Kind::InboxRelays`](crate::Kind::InboxRelays)!
pub fn extract_relay_list(event: &Event) -> impl Iterator<Item = &RelayUrl> {
    event.tags.iter().filter_map(|tag| {
        if let Some(TagStandard::Relay(url)) = tag.as_standardized() {
            Some(url)
        } else {
            None
        }
    })
}

/// Extracts the relay list
///
/// This function doesn't verify if the event kind is [`Kind::InboxRelays`](crate::Kind::InboxRelays)!
pub fn extract_owned_relay_list(event: Event) -> impl Iterator<Item = RelayUrl> {
    event.tags.into_iter().filter_map(|tag| {
        if let Some(TagStandard::Relay(url)) = tag.to_standardized() {
            Some(url)
        } else {
            None
        }
    })
}
