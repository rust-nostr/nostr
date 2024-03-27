// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect (NIP46)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

pub mod client;
pub mod error;
pub mod signer;

pub use self::client::Nip46Signer;
pub use self::error::Error;
pub use self::signer::{NostrConnectRemoteSigner, NostrConnectSignerActions};
