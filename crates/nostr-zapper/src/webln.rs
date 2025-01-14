// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! WebLN zapper backend for Nostr apps

#![cfg_attr(not(target_arch = "wasm32"), allow(unused))]

use std::ops::Deref;

use nostr::util::BoxedFuture;
use webln::WebLN;

use crate::{NostrZapper, ZapperBackend, ZapperError};

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

impl NostrZapper for WebLNZapper {
    fn backend(&self) -> ZapperBackend {
        ZapperBackend::WebLN
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn pay(&self, _invoice: String) -> BoxedFuture<Result<(), ZapperError>> {
        unreachable!("this method is supported only in WASM")
    }

    #[cfg(target_arch = "wasm32")]
    fn pay(&self, invoice: String) -> BoxedFuture<Result<(), ZapperError>> {
        Box::pin(async move {
            self.inner
                .send_payment(&invoice)
                .await
                .map_err(ZapperError::backend)?;
            Ok(())
        })
    }
}
