// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIPs
//!
//! See all at <https://github.com/nostr-protocol/nips>

#[cfg(feature = "nip04")]
pub mod nip04;
#[cfg(all(feature = "std", feature = "nip05"))]
pub mod nip05;
#[cfg(feature = "nip06")]
pub mod nip06;
#[cfg(all(feature = "std", feature = "nip11"))]
pub mod nip11;
pub mod nip13;
pub mod nip19;
pub mod nip21;
pub mod nip26;
pub mod nip33;
#[cfg(feature = "nip44")]
pub mod nip44;
#[cfg(all(feature = "std", feature = "nip46"))]
pub mod nip46;
#[cfg(feature = "nip47")]
pub mod nip47;
pub mod nip48;
pub mod nip53;
pub mod nip57;
pub mod nip58;
pub mod nip65;
pub mod nip94;
pub mod nip98;
