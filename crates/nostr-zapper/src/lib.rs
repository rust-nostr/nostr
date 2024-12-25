// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Zapper

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)]
#![allow(clippy::arc_with_non_send_sync)]

use std::fmt;
use std::sync::Arc;

pub extern crate nostr;

use async_trait::async_trait;
use nostr::prelude::*;

pub mod error;
pub mod prelude;
#[cfg(feature = "webln")]
mod webln;

pub use self::error::ZapperError;
#[cfg(feature = "webln")]
pub use self::webln::WebLNZapper;

/// Backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZapperBackend {
    /// WebLN
    WebLN,
    /// Nostr Wallet Connect
    NWC,
    /// Custom
    Custom(String),
}

/// A type-erased [`NostrZapper`].
pub type DynNostrZapper = dyn NostrZapper;

/// A type that can be type-erased into `Arc<dyn NostrZapper>`.
pub trait IntoNostrZapper {
    #[doc(hidden)]
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper>;
}

impl IntoNostrZapper for Arc<DynNostrZapper> {
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper> {
        self
    }
}

impl<T> IntoNostrZapper for T
where
    T: NostrZapper + Sized + 'static,
{
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper> {
        Arc::new(self)
    }
}

impl<T> IntoNostrZapper for Arc<T>
where
    T: NostrZapper + 'static,
{
    fn into_nostr_zapper(self) -> Arc<DynNostrZapper> {
        self
    }
}

/// Nostr Database
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrZapper: fmt::Debug + Send + Sync {
    /// Name of the backend zapper used (ex. WebLN, NWC, ...)
    fn backend(&self) -> ZapperBackend;

    /// Pay invoice
    async fn pay(&self, invoice: String) -> Result<(), ZapperError>;
}
