// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_js::nips::nip11::JsRelayInformationDocument;
use nostr_sdk::prelude::*;
use nostr_sdk::relay::Relay;
use wasm_bindgen::prelude::*;

pub mod options;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "JsRelay[]")]
    pub type JsRelayArray;
}

#[wasm_bindgen(js_name = Relay)]
pub struct JsRelay {
    inner: Relay,
}

impl From<Relay> for JsRelay {
    fn from(relay: Relay) -> Self {
        Self { inner: relay }
    }
}

#[wasm_bindgen(js_name = RelayStatus)]
pub enum JsRelayStatus {
    /// Relay initialized
    Initialized,
    /// Pending
    Pending,
    /// Connecting
    Connecting,
    /// Relay connected
    Connected,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Stop
    Stopped,
    /// Relay completely disconnected
    Terminated,
}

impl From<RelayStatus> for JsRelayStatus {
    fn from(status: RelayStatus) -> Self {
        match status {
            RelayStatus::Initialized => Self::Initialized,
            RelayStatus::Pending => Self::Pending,
            RelayStatus::Connecting => Self::Connecting,
            RelayStatus::Connected => Self::Connected,
            RelayStatus::Disconnected => Self::Disconnected,
            RelayStatus::Stopped => Self::Stopped,
            RelayStatus::Terminated => Self::Terminated,
        }
    }
}

#[wasm_bindgen(js_class = Relay)]
impl JsRelay {
    /// Get relay url
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    // Get status
    pub async fn status(&self) -> JsRelayStatus {
        self.inner.status().await.into()
    }

    /// Check if relay is connected
    #[wasm_bindgen(js_name = isConnected)]
    pub async fn is_connected(&self) -> bool {
        self.inner.is_connected().await
    }

    /// Get `RelayInformationDocument`
    pub async fn document(&self) -> JsRelayInformationDocument {
        self.inner.document().await.into()
    }
}
