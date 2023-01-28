// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]

//! Rust implementation of the Nostr protocol.

pub use bitcoin::hashes;
pub use bitcoin::secp256k1;
#[cfg(feature = "base")]
pub use url::{self, Url};

#[cfg(feature = "default")]
mod doctest;
#[cfg(feature = "base")]
pub mod event;
pub mod key;
#[cfg(feature = "base")]
pub mod message;
pub mod nips;
pub mod prelude;
#[cfg(feature = "base")]
pub mod types;
#[deprecated]
pub mod util;

#[cfg(feature = "base")]
pub use self::event::{Event, EventBuilder, EventId, Kind, Tag};
pub use self::key::Keys;
#[cfg(feature = "base")]
pub use self::message::{ClientMessage, RelayMessage, SubscriptionFilter, SubscriptionId};
#[cfg(feature = "base")]
pub use self::types::{Contact, Entity, Metadata, Profile, Timestamp};

/// Result
pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
