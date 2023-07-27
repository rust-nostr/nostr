// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::event::tag;

pub enum TagKind {
    Known { known: TagKindKnown },
    Unknown { unknown: String },
}

pub enum TagKindKnown {
    /// Public key
    P,
    /// Event id
    E,
    /// Reference (URL, etc.)
    R,
    /// Hashtag
    T,
    /// Geohash
    G,
    /// Identifier
    D,
    /// Referencing and tagging
    A,
    /// External Identities
    I,
    /// MIME type
    M,
    /// Absolute URL
    U,
    /// SHA256
    X,
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
    /// HTTP Method Request
    Method,
    /// Payload HASH
    Payload,
}

impl From<tag::TagKind> for TagKind {
    fn from(value: tag::TagKind) -> Self {
        match value {
            tag::TagKind::P => Self::Known {
                known: TagKindKnown::P,
            },
            tag::TagKind::E => Self::Known {
                known: TagKindKnown::E,
            },
            tag::TagKind::R => Self::Known {
                known: TagKindKnown::R,
            },
            tag::TagKind::T => Self::Known {
                known: TagKindKnown::T,
            },
            tag::TagKind::G => Self::Known {
                known: TagKindKnown::G,
            },
            tag::TagKind::D => Self::Known {
                known: TagKindKnown::D,
            },
            tag::TagKind::A => Self::Known {
                known: TagKindKnown::A,
            },
            tag::TagKind::I => Self::Known {
                known: TagKindKnown::I,
            },
            tag::TagKind::M => Self::Known {
                known: TagKindKnown::M,
            },
            tag::TagKind::U => Self::Known {
                known: TagKindKnown::U,
            },
            tag::TagKind::X => Self::Known {
                known: TagKindKnown::X,
            },
            tag::TagKind::Relay => Self::Known {
                known: TagKindKnown::RelayUrl,
            },
            tag::TagKind::Nonce => Self::Known {
                known: TagKindKnown::Nonce,
            },
            tag::TagKind::Delegation => Self::Known {
                known: TagKindKnown::Delegation,
            },
            tag::TagKind::ContentWarning => Self::Known {
                known: TagKindKnown::ContentWarning,
            },
            tag::TagKind::Expiration => Self::Known {
                known: TagKindKnown::Expiration,
            },
            tag::TagKind::Subject => Self::Known {
                known: TagKindKnown::Subject,
            },
            tag::TagKind::Challenge => Self::Known {
                known: TagKindKnown::Challenge,
            },
            tag::TagKind::Title => Self::Known {
                known: TagKindKnown::Title,
            },
            tag::TagKind::Image => Self::Known {
                known: TagKindKnown::Image,
            },
            tag::TagKind::Thumb => Self::Known {
                known: TagKindKnown::Thumb,
            },
            tag::TagKind::Summary => Self::Known {
                known: TagKindKnown::Summary,
            },
            tag::TagKind::PublishedAt => Self::Known {
                known: TagKindKnown::PublishedAt,
            },
            tag::TagKind::Description => Self::Known {
                known: TagKindKnown::Description,
            },
            tag::TagKind::Bolt11 => Self::Known {
                known: TagKindKnown::Bolt11,
            },
            tag::TagKind::Preimage => Self::Known {
                known: TagKindKnown::Preimage,
            },
            tag::TagKind::Relays => Self::Known {
                known: TagKindKnown::Relays,
            },
            tag::TagKind::Amount => Self::Known {
                known: TagKindKnown::Amount,
            },
            tag::TagKind::Lnurl => Self::Known {
                known: TagKindKnown::Lnurl,
            },
            tag::TagKind::Name => Self::Known {
                known: TagKindKnown::Name,
            },
            tag::TagKind::Url => Self::Known {
                known: TagKindKnown::Url,
            },
            tag::TagKind::Aes256Gcm => Self::Known {
                known: TagKindKnown::Aes256Gcm,
            },
            tag::TagKind::Size => Self::Known {
                known: TagKindKnown::Size,
            },
            tag::TagKind::Dim => Self::Known {
                known: TagKindKnown::Dim,
            },
            tag::TagKind::Magnet => Self::Known {
                known: TagKindKnown::Magnet,
            },
            tag::TagKind::Blurhash => Self::Known {
                known: TagKindKnown::Blurhash,
            },
            tag::TagKind::Streaming => Self::Known {
                known: TagKindKnown::Streaming,
            },
            tag::TagKind::Recording => Self::Known {
                known: TagKindKnown::Recording,
            },
            tag::TagKind::Starts => Self::Known {
                known: TagKindKnown::Starts,
            },
            tag::TagKind::Ends => Self::Known {
                known: TagKindKnown::Ends,
            },
            tag::TagKind::Status => Self::Known {
                known: TagKindKnown::Status,
            },
            tag::TagKind::CurrentParticipants => Self::Known {
                known: TagKindKnown::CurrentParticipants,
            },
            tag::TagKind::TotalParticipants => Self::Known {
                known: TagKindKnown::TotalParticipants,
            },
            tag::TagKind::Method => Self::Known {
                known: TagKindKnown::Method,
            },
            tag::TagKind::Payload => Self::Known {
                known: TagKindKnown::Payload,
            },
            tag::TagKind::Custom(unknown) => Self::Unknown { unknown },
        }
    }
}
