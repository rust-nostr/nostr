// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]

// External crates
pub use ::url::*;
#[cfg(feature = "nip19")]
pub use bech32::*;
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
pub use crate::{Result, SECP256K1};

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
#[cfg(feature = "nip21")]
pub use crate::nips::nip21::*;
pub use crate::nips::nip26::*;
pub use crate::nips::nip33::*;
#[cfg(feature = "nip46")]
pub use crate::nips::nip46::*;
#[cfg(feature = "nip47")]
pub use crate::nips::nip47::*;
pub use crate::nips::nip53::*;
pub use crate::nips::nip57::*;
pub use crate::nips::nip58::*;
pub use crate::nips::nip65::*;
pub use crate::nips::nip94::*;
pub use crate::nips::nip98::*;
