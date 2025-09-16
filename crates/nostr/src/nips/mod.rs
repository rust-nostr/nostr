// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIPs
//!
//! See all at <https://github.com/nostr-protocol/nips>

pub mod nip01;
pub mod nip02;
#[cfg(feature = "nip04")]
pub mod nip04;
pub mod nip05;
#[cfg(feature = "nip06")]
pub mod nip06;
pub mod nip09;
pub mod nip10;
pub mod nip11;
pub mod nip13;
pub mod nip15;
pub mod nip17;
pub mod nip19;
pub mod nip21;
pub mod nip22;
pub mod nip25;
pub mod nip34;
pub mod nip35;
pub mod nip38;
pub mod nip39;
pub mod nip42;
#[cfg(feature = "nip44")]
pub mod nip44;
#[cfg(all(feature = "std", feature = "nip46"))]
pub mod nip46;
#[cfg(feature = "nip47")]
pub mod nip47;
pub mod nip48;
#[cfg(feature = "nip49")]
pub mod nip49;
pub mod nip51;
pub mod nip53;
pub mod nip56;
#[cfg(feature = "nip57")]
pub mod nip57;
pub mod nip58;
#[cfg(feature = "nip59")]
pub mod nip59;
pub mod nip62;
pub mod nip65;
pub mod nip73;
pub mod nip88;
pub mod nip90;
pub mod nip94;
#[cfg(all(feature = "std", feature = "nip96"))]
pub mod nip96;
#[cfg(feature = "nip98")]
pub mod nip98;
pub mod nipc0;
pub mod nipb0;
