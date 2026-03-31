// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP44: Versioned Encryption
//!
//! <https://github.com/nostr-protocol/nips/blob/master/44.md>

#[cfg(feature = "nip44")]
mod r#impl;
mod traits;
#[cfg(feature = "nip44")]
pub mod v2;

#[cfg(feature = "nip44")]
pub use self::r#impl::*;
pub use self::traits::*;
