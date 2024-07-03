// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip10;
use uniffi::Enum;

/// Marker
#[derive(Enum, o2o::o2o)]
#[map_owned(nip10::Marker)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Mention
    Mention,
    /// Custom
    #[type_hint(as())]
    Custom { custom: String },
}
