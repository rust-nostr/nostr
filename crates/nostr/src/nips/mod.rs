// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIPs
//!
//! See all at <https://github.com/nostr-protocol/nips>

#[cfg(feature = "nip04")]
pub mod nip04;
#[cfg(feature = "nip05")]
pub mod nip05;
#[cfg(feature = "nip06")]
pub mod nip06;
#[cfg(feature = "nip11")]
pub mod nip11;
#[cfg(feature = "nip13")]
pub mod nip13;
#[cfg(feature = "nip19")]
pub mod nip19;
#[cfg(feature = "nip26")]
pub mod nip26;
#[cfg(feature = "nip46")]
pub mod nip46;
#[cfg(all(feature = "nip65", feature = "base"))]
pub mod nip65;
