// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

// External crates
pub use async_utility::futures_util::StreamExt; // Needed for `RelayPool::stream_events_of`
pub use nostr::prelude::*;
pub use nostr_database::*;

// Internal modules
pub use crate::policy::*;
pub use crate::pool::constants::*;
pub use crate::pool::options::*;
pub use crate::pool::{self, *};
pub use crate::relay::{self, *};
pub use crate::stream::*;
pub use crate::*;
