// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! WebLN zapper backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints, clippy::arc_with_non_send_sync)]
#![cfg_attr(not(target_arch = "wasm32"), allow(unused))]

pub extern crate nostr_zapper as zapper;
pub extern crate webln;

use std::ops::Deref;

#[cfg(target_arch = "wasm32")]
use nostr_zapper::NostrZapper;
use nostr_zapper::{ZapperBackend, ZapperError};
use webln::WebLN;

/// [WebLN] zapper
#[derive(Debug, Clone)]
pub struct WebLNZapper {
    inner: WebLN,
}

impl Deref for WebLNZapper {
    type Target = WebLN;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl WebLNZapper {
    /// New [WebLN] zapper
    ///
    /// Internally, automatically call `webln.enable()`.
    pub async fn new() -> Result<Self, ZapperError> {
        let inner = WebLN::new().map_err(ZapperError::backend)?;
        inner.enable().await.map_err(ZapperError::backend)?;
        Ok(Self { inner })
    }
}

#[cfg(target_arch = "wasm32")]
macro_rules! impl_nostr_zapper {
    ({ $($body:tt)* }) => {
        #[nostr_zapper::async_trait(?Send)]
        impl NostrZapper for WebLNZapper {
            $($body)*
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! impl_nostr_zapper {
    ({ $($body:tt)* }) => {
        impl WebLNZapper {
            $($body)*
        }
    };
}

impl_nostr_zapper!({
    fn backend(&self) -> ZapperBackend {
        ZapperBackend::WebLN
    }

    async fn pay(&self, invoice: String) -> Result<(), ZapperError> {
        self.inner
            .send_payment(&invoice)
            .await
            .map_err(ZapperError::backend)?;
        Ok(())
    }
});
