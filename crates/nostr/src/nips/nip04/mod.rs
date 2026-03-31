// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP04: Encrypted Direct Message (deprecated in favor of NIP17)
//!
//! <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP17!</div>
//!
//! <https://github.com/nostr-protocol/nips/blob/master/04.md>

#[cfg(feature = "nip04")]
mod r#impl;
mod traits;

#[cfg(feature = "nip04")]
pub use self::r#impl::*;
pub use self::traits::*;
