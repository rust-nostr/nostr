// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]

// External crates
pub use nostr::prelude::*;

// Internal modules
pub use crate::client::*;
pub use crate::relay::*;
pub use crate::*;
