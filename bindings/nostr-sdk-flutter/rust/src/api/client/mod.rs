// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::Result;
use flutter_rust_bridge::frb;
use nostr_sdk::prelude::*;

use super::protocol::event::_Event;

#[frb(name = "Client")]
pub struct _Client {
    inner: Client,
}

impl _Client {
    #[frb(sync)]
    pub fn default() -> Self {
        Self {
            inner: Client::default(),
        }
    }

    pub async fn add_relay(&self, url: &str) -> Result<bool> {
        Ok(self.inner.add_relay(url).await?)
    }

    pub async fn connect(&self) {
        self.inner.connect().await
    }

    pub async fn send_event(&self, event: _Event) -> Result<String> {
        let output = self.inner.send_event(event.inner).await?;
        Ok(output.id().to_string())
    }
}
