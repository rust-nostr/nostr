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
pub mod nip13;
#[cfg(feature = "nip19")]
pub mod nip19;
pub mod nip26;
#[cfg(feature = "nip46")]
pub mod nip46;
pub mod nip58;
pub mod nip65;
