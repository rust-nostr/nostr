// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip48;
use uniffi::Enum;

/// NIP48 Proxy Protocol
#[derive(Enum)]
pub enum Protocol {
    /// ActivityPub
    ActivityPub,
    /// AT Protocol
    ATProto,
    /// Rss
    Rss,
    /// Web
    Web,
    /// Custom
    Custom { custom: String },
}

impl From<Protocol> for nip48::Protocol {
    fn from(value: Protocol) -> Self {
        match value {
            Protocol::ActivityPub => Self::ActivityPub,
            Protocol::ATProto => Self::ATProto,
            Protocol::Rss => Self::Rss,
            Protocol::Web => Self::Web,
            Protocol::Custom { custom } => Self::Custom(custom),
        }
    }
}

impl From<nip48::Protocol> for Protocol {
    fn from(value: nip48::Protocol) -> Self {
        match value {
            nip48::Protocol::ActivityPub => Self::ActivityPub,
            nip48::Protocol::ATProto => Self::ATProto,
            nip48::Protocol::Rss => Self::Rss,
            nip48::Protocol::Web => Self::Web,
            nip48::Protocol::Custom(custom) => Self::Custom { custom },
        }
    }
}
