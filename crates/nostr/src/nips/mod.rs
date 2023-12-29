// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIPs
//!
//! See all at <https://github.com/nostr-protocol/nips>

pub mod nip01;
#[cfg(feature = "nip04")]
pub mod nip04;
#[cfg(all(feature = "std", feature = "nip05"))]
pub mod nip05;
#[cfg(feature = "nip06")]
pub mod nip06;
#[cfg(all(feature = "std", feature = "nip11"))]
pub mod nip11;
pub mod nip13;
pub mod nip15;
pub mod nip19;
pub mod nip21;
pub mod nip26;
#[deprecated(since = "0.26.0", note = "moved to `nip01`")]
pub mod nip33;
#[cfg(feature = "nip44")]
pub mod nip44;
#[cfg(all(feature = "std", feature = "nip46"))]
pub mod nip46;
#[cfg(feature = "nip47")]
pub mod nip47;
pub mod nip48;
pub mod nip53;
#[cfg(feature = "nip57")]
pub mod nip57;
pub mod nip58;
pub mod nip65;
pub mod nip90;
pub mod nip94;
pub mod nip98;
