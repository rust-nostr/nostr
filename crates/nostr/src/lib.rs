// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub use bitcoin::hashes;
pub use bitcoin::hashes::sha256::Hash as Sha256Hash;
pub use bitcoin::secp256k1;
#[cfg(feature = "event")]
pub use url::{self, Url};

#[cfg(feature = "full")]
mod doctest;
#[cfg(feature = "event")]
pub mod event;
pub mod key;
#[cfg(feature = "event")]
pub mod message;
pub mod prelude;
#[cfg(feature = "event")]
pub mod types;
pub mod util;

#[cfg(feature = "event")]
pub use self::event::{Event, EventBuilder, Kind, Tag};
pub use self::key::Keys;
#[cfg(feature = "event")]
pub use self::message::{ClientMessage, RelayMessage, SubscriptionFilter};
#[cfg(feature = "event")]
pub use self::types::{Contact, Entity, Metadata};

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
