// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Prelude

// External crates
pub use nostr::prelude::*;

// Internal modules
#[cfg(feature = "rest-api")]
pub use crate::client::rest::*;
#[cfg(feature = "websocket")]
pub use crate::client::websocket::*;
#[cfg(feature = "websocket")]
pub use crate::relay::*;
pub use crate::*;
