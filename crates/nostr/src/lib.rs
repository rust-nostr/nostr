// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Rust implementation of the Nostr protocol.

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(not(feature = "std"), no_std)]
//#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(error_in_core))]
#![cfg_attr(
    feature = "default",
    doc = include_str!("../README.md")
)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("at least one of the `std` or `alloc` features must be enabled");

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[macro_use]
pub extern crate alloc;

#[macro_use]
extern crate serde;

#[cfg(feature = "nip06")]
pub use bip39;
pub use bitcoin;
pub use bitcoin::bech32;
pub use bitcoin::hashes;
pub use bitcoin::secp256k1;
pub use serde_json;
pub use url_fork::{self as url, Url};

pub mod event;
pub mod key;
pub mod message;
pub mod nips;
pub mod prelude;
pub mod types;
pub mod util;

pub use self::event::tag::{
    ExternalIdentity, HttpMethod, Identity, ImageDimensions, Marker, RelayMetadata, Report, Tag,
    TagKind,
};
pub use self::event::{Event, EventBuilder, EventId, Kind, UnsignedEvent};
pub use self::key::Keys;
pub use self::message::{Alphabet, ClientMessage, Filter, RelayMessage, SubscriptionId};
pub use self::nips::nip19::{FromBech32, ToBech32};
pub use self::types::{ChannelId, Contact, Entity, Metadata, Profile, Timestamp, UncheckedUrl};
#[cfg(feature = "std")]
pub use self::util::SECP256K1;

/// Result
#[cfg(feature = "std")]
pub type Result<T, E = alloc::boxed::Box<dyn std::error::Error>> = std::result::Result<T, E>;
