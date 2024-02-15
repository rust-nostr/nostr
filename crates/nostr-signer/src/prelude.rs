// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

#[cfg(feature = "nip46")]
pub use crate::nip46::Nip46Signer;
pub use crate::*;
