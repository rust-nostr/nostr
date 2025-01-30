// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::borrow::Cow;
use std::sync::Arc;

use nostr::event::tag;
use uniffi::Enum;

use crate::protocol::filter::SingleLetterTag;

#[derive(Enum)]
pub enum TagKind {
    /// Human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt,
    /// Client
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/89.md>
    Client,
    Clone,
    /// Commit
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    Commit,
    /// Maintainers
    Maintainers,
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// Relay
    RelayUrl,
    /// Nonce
    Nonce,
    /// Delegation
    Delegation,
    /// Content warning
    ContentWarning,
    /// Expiration
    Expiration,
    /// Subject
    Subject,
    /// Auth challenge
    Challenge,
    /// Title (NIP23)
    Title,
    /// Image (NIP23)
    Image,
    /// Thumbnail
    Thumb,
    /// Summary (NIP23)
    Summary,
    /// PublishedAt (NIP23)
    PublishedAt,
    /// Description (NIP57)
    Description,
    /// Bolt11 Invoice (NIP57)
    Bolt11,
    /// Preimage (NIP57)
    Preimage,
    /// Relays (NIP57)
    Relays,
    /// Amount (NIP57)
    Amount,
    /// Lnurl (NIP57)
    Lnurl,
    /// MLS Protocol Version (NIP104)
    MlsProtocolVersion,
    /// MLS Cipher Suite (NIP104)
    MlsCiphersuite,
    /// MLS Extensions (NIP104)
    MlsExtensions,
    /// Name tag
    Name,
    /// Url
    Url,
    /// AES 256 GCM
    Aes256Gcm,
    /// Size of file in bytes
    Size,
    /// Size of file in pixels
    Dim,
    File,
    /// Magnet
    Magnet,
    /// Blurhash
    Blurhash,
    /// Streaming
    Streaming,
    /// Recording
    Recording,
    /// Starts
    Starts,
    /// Ends
    Ends,
    /// Status
    Status,
    /// Current participants
    CurrentParticipants,
    /// Total participants
    TotalParticipants,
    Tracker,
    /// HTTP Method Request
    Method,
    /// Payload HASH
    Payload,
    Anon,
    Proxy,
    Emoji,
    /// Encrypted
    Encrypted,
    Request,
    Web,
    Word,
    SingleLetter {
        single_letter: Arc<SingleLetterTag>,
    },
    Unknown {
        unknown: String,
    },
}

impl From<tag::TagKind<'_>> for TagKind {
    fn from(value: tag::TagKind<'_>) -> Self {
        match value {
            tag::TagKind::Alt => Self::Alt,
            tag::TagKind::Client => Self::Client,
            tag::TagKind::Clone => Self::Clone,
            tag::TagKind::Commit => Self::Commit,
            tag::TagKind::Maintainers => Self::Maintainers,
            tag::TagKind::Protected => Self::Protected,
            tag::TagKind::Relay => Self::RelayUrl,
            tag::TagKind::Nonce => Self::Nonce,
            tag::TagKind::Delegation => Self::Delegation,
            tag::TagKind::ContentWarning => Self::ContentWarning,
            tag::TagKind::Expiration => Self::Expiration,
            tag::TagKind::Subject => Self::Subject,
            tag::TagKind::Challenge => Self::Challenge,
            tag::TagKind::Title => Self::Title,
            tag::TagKind::Image => Self::Image,
            tag::TagKind::Thumb => Self::Thumb,
            tag::TagKind::Summary => Self::Summary,
            tag::TagKind::PublishedAt => Self::PublishedAt,
            tag::TagKind::Description => Self::Description,
            tag::TagKind::Bolt11 => Self::Bolt11,
            tag::TagKind::Preimage => Self::Preimage,
            tag::TagKind::Relays => Self::Relays,
            tag::TagKind::Amount => Self::Amount,
            tag::TagKind::Lnurl => Self::Lnurl,
            tag::TagKind::Name => Self::Name,
            tag::TagKind::Url => Self::Url,
            tag::TagKind::Aes256Gcm => Self::Aes256Gcm,
            tag::TagKind::Size => Self::Size,
            tag::TagKind::Dim => Self::Dim,
            tag::TagKind::File => Self::File,
            tag::TagKind::Magnet => Self::Magnet,
            tag::TagKind::Blurhash => Self::Blurhash,
            tag::TagKind::Streaming => Self::Streaming,
            tag::TagKind::Recording => Self::Recording,
            tag::TagKind::Starts => Self::Starts,
            tag::TagKind::Ends => Self::Ends,
            tag::TagKind::Status => Self::Status,
            tag::TagKind::CurrentParticipants => Self::CurrentParticipants,
            tag::TagKind::TotalParticipants => Self::TotalParticipants,
            tag::TagKind::Tracker => Self::Tracker,
            tag::TagKind::Method => Self::Method,
            tag::TagKind::Payload => Self::Payload,
            tag::TagKind::Anon => Self::Anon,
            tag::TagKind::Proxy => Self::Proxy,
            tag::TagKind::Emoji => Self::Emoji,
            tag::TagKind::Encrypted => Self::Encrypted,
            tag::TagKind::Request => Self::Request,
            tag::TagKind::Web => Self::Web,
            tag::TagKind::Word => Self::Word,
            tag::TagKind::MlsProtocolVersion => Self::MlsProtocolVersion,
            tag::TagKind::MlsCiphersuite => Self::MlsCiphersuite,
            tag::TagKind::MlsExtensions => Self::MlsExtensions,
            tag::TagKind::SingleLetter(single_letter) => Self::SingleLetter {
                single_letter: Arc::new(single_letter.into()),
            },
            tag::TagKind::Custom(unknown) => Self::Unknown {
                unknown: unknown.to_string(),
            },
        }
    }
}

impl From<TagKind> for tag::TagKind<'_> {
    fn from(value: TagKind) -> Self {
        match value {
            TagKind::Alt => Self::Alt,
            TagKind::Client => Self::Client,
            TagKind::Clone => Self::Clone,
            TagKind::Commit => Self::Commit,
            TagKind::Maintainers => Self::Maintainers,
            TagKind::Protected => Self::Protected,
            TagKind::RelayUrl => Self::Relay,
            TagKind::Nonce => Self::Nonce,
            TagKind::Delegation => Self::Delegation,
            TagKind::ContentWarning => Self::ContentWarning,
            TagKind::Expiration => Self::Expiration,
            TagKind::Subject => Self::Subject,
            TagKind::Challenge => Self::Challenge,
            TagKind::Title => Self::Title,
            TagKind::Image => Self::Image,
            TagKind::Thumb => Self::Thumb,
            TagKind::Summary => Self::Summary,
            TagKind::PublishedAt => Self::PublishedAt,
            TagKind::Description => Self::Description,
            TagKind::Bolt11 => Self::Bolt11,
            TagKind::Preimage => Self::Preimage,
            TagKind::Relays => Self::Relays,
            TagKind::Amount => Self::Amount,
            TagKind::Lnurl => Self::Lnurl,
            TagKind::Name => Self::Name,
            TagKind::Url => Self::Url,
            TagKind::Aes256Gcm => Self::Aes256Gcm,
            TagKind::Size => Self::Size,
            TagKind::Dim => Self::Dim,
            TagKind::File => Self::File,
            TagKind::Magnet => Self::Magnet,
            TagKind::Blurhash => Self::Blurhash,
            TagKind::Streaming => Self::Streaming,
            TagKind::Recording => Self::Recording,
            TagKind::Starts => Self::Starts,
            TagKind::Ends => Self::Ends,
            TagKind::Status => Self::Status,
            TagKind::CurrentParticipants => Self::CurrentParticipants,
            TagKind::TotalParticipants => Self::TotalParticipants,
            TagKind::Tracker => Self::Tracker,
            TagKind::Method => Self::Method,
            TagKind::Payload => Self::Payload,
            TagKind::Anon => Self::Anon,
            TagKind::Proxy => Self::Proxy,
            TagKind::Emoji => Self::Emoji,
            TagKind::Encrypted => Self::Encrypted,
            TagKind::Request => Self::Request,
            TagKind::Web => Self::Web,
            TagKind::Word => Self::Word,
            TagKind::MlsProtocolVersion => Self::MlsProtocolVersion,
            TagKind::MlsCiphersuite => Self::MlsCiphersuite,
            TagKind::MlsExtensions => Self::MlsExtensions,
            TagKind::SingleLetter { single_letter } => Self::SingleLetter(**single_letter),
            TagKind::Unknown { unknown } => Self::Custom(Cow::Owned(unknown)),
        }
    }
}

/// Convert tag kind to string
#[uniffi::export]
pub fn tag_kind_to_string(kind: TagKind) -> String {
    let kind: tag::TagKind<'_> = kind.into();
    kind.to_string()
}
