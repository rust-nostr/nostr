// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

pub use nostr::prelude::*;
// Re-export for tests and examples
#[cfg(any(test, feature = "test-utils"))]
pub use nostr_mls_memory_storage::NostrMlsMemoryStorage;
pub use openmls::prelude::*;

// Re-export nostr-mls-storage types and traits
pub use nostr_mls_storage::groups::{types as group_types, GroupStorage};
pub use nostr_mls_storage::messages::{types as message_types, MessageStorage};
pub use nostr_mls_storage::welcomes::{types as welcome_types, WelcomeStorage};
pub use nostr_mls_storage::{Backend, NostrMlsStorageProvider};

pub use crate::extension::*;
pub use crate::groups::*;
pub use crate::welcomes::*;
pub use crate::*;
