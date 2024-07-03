// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip48;
use uniffi::Enum;

/// NIP48 Proxy Protocol
#[derive(Enum, o2o::o2o)]
#[map_owned(nip48::Protocol)]
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
    #[type_hint(as ())]
    Custom { custom: String },
}
