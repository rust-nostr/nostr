// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip10;
use uniffi::Enum;

/// Marker
#[derive(Enum)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Mention
    Mention,
    /// Custom
    Custom { custom: String },
}

impl From<Marker> for nip10::Marker {
    fn from(value: Marker) -> Self {
        match value {
            Marker::Root => Self::Root,
            Marker::Reply => Self::Reply,
            Marker::Mention => Self::Mention,
            Marker::Custom { custom } => Self::Custom(custom),
        }
    }
}

impl From<nip10::Marker> for Marker {
    fn from(value: nip10::Marker) -> Self {
        match value {
            nip10::Marker::Root => Self::Root,
            nip10::Marker::Reply => Self::Reply,
            nip10::Marker::Mention => Self::Mention,
            nip10::Marker::Custom(custom) => Self::Custom { custom },
        }
    }
}
