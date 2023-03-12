// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client

#[cfg(feature = "rest-api")]
#[cfg_attr(docsrs, doc(cfg(feature = "rest-api")))]
pub mod rest;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

#[cfg(feature = "rest-api")]
pub use self::rest::*;
#[cfg(feature = "websocket")]
pub use self::websocket::*;
