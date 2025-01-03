// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

// External crates
pub use nostr::prelude::*;

// Internal modules
pub use crate::collections::events::*;
pub use crate::events::*;
pub use crate::memory::{self, *};
pub use crate::*;
