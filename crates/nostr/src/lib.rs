// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(error_in_core))]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

//! Rust implementation of the Nostr protocol.

#![cfg_attr(
    feature = "default",
    doc = include_str!("../README.md")
)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[macro_use]
/// Serde crate
pub extern crate serde;

#[cfg(feature = "nip19")]
pub use bech32;
#[cfg(feature = "nip06")]
pub use bip39;
#[cfg(feature = "nip06")]
pub use bitcoin;
pub use bitcoin_hashes as hashes;
pub use secp256k1;
#[cfg(feature = "std")]
pub use secp256k1::SECP256K1;
pub use serde_json;

#[cfg(feature = "std")]
pub use url;
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate url_no_std as url;

pub use url::Url;

pub mod event;
pub mod key;
pub mod message;
pub mod nips;
pub mod prelude;
pub mod types;

pub use self::event::{Event, EventBuilder, EventId, Kind, Tag, TagKind, UnsignedEvent};
pub use self::key::Keys;
pub use self::message::{ClientMessage, Filter, RelayMessage, SubscriptionId};
pub use self::types::{ChannelId, Contact, Entity, Metadata, Profile, Timestamp, UncheckedUrl};

/// Result
#[cfg(feature = "std")]
pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
/// Result
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub type Result<T, E = Box<dyn core::error::Error>> = core::result::Result<T, E>;
