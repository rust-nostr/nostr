// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_relay_builder::mock;
use uniffi::Object;

use crate::error::Result;

/// A mock relay for (unit) tests.
#[derive(Object)]
pub struct MockRelay {
    inner: mock::MockRelay,
}

#[uniffi::export(async_runtime = "tokio")]
impl MockRelay {
    #[uniffi::constructor]
    pub async fn run() -> Result<Self> {
        Ok(Self {
            inner: mock::MockRelay::run().await?,
        })
    }

    /// Get url
    pub fn url(&self) -> String {
        self.inner.url()
    }

    /// Shutdown relay
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }

    // `#[uniffi::export(async_runtime = "tokio")]` require an async method!
    async fn _none(&self) {}
}
