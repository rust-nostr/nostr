// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Rust implementation of the Nostr protocol.

#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]
#![cfg_attr(target_arch = "wasm32", deny(unsafe_code))]
#![cfg_attr(test, allow(missing_docs))]
#![cfg_attr(not(test), warn(missing_docs))]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(bench, feature(test))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(all(feature = "std", feature = "all-nips"), doc = include_str!("../README.md"))]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("at least one of the `std` or `alloc` features must be enabled");

#[cfg(bench)]
extern crate test;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate serde;
pub extern crate bitcoin_hashes as hashes;
pub extern crate secp256k1;

#[doc(hidden)]
#[cfg(any(feature = "nip04", feature = "nip44"))]
pub use base64;
#[doc(hidden)]
#[cfg(feature = "nip06")]
pub use bip39;
#[doc(hidden)]
pub use serde_json;

pub mod event;
pub mod filter;
pub mod key;
pub mod message;
pub mod nips;
pub mod prelude;
pub mod signer;
pub mod types;
pub mod util;

#[doc(hidden)]
pub use self::event::tag::{Tag, TagKind, TagStandard, Tags};
#[doc(hidden)]
pub use self::event::{Event, EventBuilder, EventId, Kind, UnsignedEvent};
#[doc(hidden)]
pub use self::filter::{Alphabet, Filter, SingleLetterTag};
#[doc(hidden)]
pub use self::key::{Keys, PublicKey, SecretKey};
#[doc(hidden)]
pub use self::message::{ClientMessage, RelayMessage, SubscriptionId};
#[doc(hidden)]
pub use self::nips::nip01::Metadata;
#[doc(hidden)]
pub use self::nips::nip19::{FromBech32, ToBech32};
#[doc(hidden)]
pub use self::signer::{NostrSigner, SignerError};
#[doc(hidden)]
pub use self::types::{ImageDimensions, RelayUrl, Timestamp, TryIntoUrl, Url};
#[doc(hidden)]
pub use self::util::JsonUtil;
#[doc(hidden)]
#[cfg(feature = "std")]
pub use self::util::SECP256K1;

/// Result
#[doc(hidden)]
#[cfg(feature = "std")]
pub type Result<T, E = alloc::boxed::Box<dyn std::error::Error>> = std::result::Result<T, E>;
