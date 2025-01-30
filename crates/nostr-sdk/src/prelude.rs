// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

// External crates
pub use nostr::prelude::*;
pub use nostr_database::prelude::*;
pub use nostr_relay_pool::prelude::*;

// Internal modules
pub use crate::client::*;
pub use crate::*;
