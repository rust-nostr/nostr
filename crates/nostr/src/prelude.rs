// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Prelude
//#![allow(ambiguous_glob_reexports)]
#![cfg_attr(
    all(not(feature = "std"), feature = "alloc"),
    allow(ambiguous_glob_reexports)
)]

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]

// External crates
#[cfg(feature = "nip19")]
pub use ::bech32::*;
pub use ::url::*;
#[cfg(feature = "nip06")]
pub use bip39::*;
#[cfg(feature = "nip06")]
pub use bitcoin::*;
pub use bitcoin_hashes::*;
pub use secp256k1::*;
pub use serde_json::*;

// Internal modules
pub use crate::event::builder::*;
pub use crate::event::id::*;
pub use crate::event::kind::*;
pub use crate::event::tag::*;
pub use crate::event::unsigned::*;
pub use crate::event::*;
pub use crate::key::*;
pub use crate::message::*;
pub use crate::types::*;
pub use crate::Result;
#[cfg(feature = "std")]
pub use crate::SECP256K1;

// NIPs
#[cfg(feature = "nip04")]
pub use crate::nips::nip04::*;
#[cfg(feature = "nip05")]
pub use crate::nips::nip05::*;
#[cfg(feature = "nip06")]
pub use crate::nips::nip06::*;
#[cfg(feature = "nip11")]
pub use crate::nips::nip11::*;
pub use crate::nips::nip13::*;
#[cfg(feature = "nip19")]
pub use crate::nips::nip19::*;
pub use crate::nips::nip26::*;
#[cfg(feature = "nip46")]
pub use crate::nips::nip46::*;
pub use crate::nips::nip65::*;
