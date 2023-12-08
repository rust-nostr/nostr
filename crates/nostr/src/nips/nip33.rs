// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP33
//!
//! <https://github.com/nostr-protocol/nips/blob/master/33.md>

#![allow(missing_docs)]

use core::ops::Deref;

pub use super::nip01::Coordinate;

#[deprecated(since = "0.26.0", note = "use `Coordinate` instead")]
pub struct ParameterizedReplaceableEvent {
    inner: Coordinate,
}

#[allow(deprecated)]
impl Deref for ParameterizedReplaceableEvent {
    type Target = Coordinate;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
