// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use wasm_bindgen::prelude::*;
use nostr_js::nips::nip11::JsRelayInformationDocument;
use nostr_sdk::prelude::*;
use nostr_sdk::relay::Relay;

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
    /// Relay connected
    Connected,
    /// Connecting
    Connecting,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Relay completly disconnected
    Terminated,
}

impl From<RelayStatus> for JsRelayStatus {
    fn from(status: RelayStatus) -> Self {
        match status {
            RelayStatus::Initialized => Self::Initialized,
            RelayStatus::Connected => Self::Connected,
            RelayStatus::Connecting => Self::Connecting,
            RelayStatus::Disconnected => Self::Disconnected,
            RelayStatus::Terminated => Self::Terminated,
        }
    }
}

#[wasm_bindgen(js_class = Relay)]
impl JsRelay {
    /// Get relay url
    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    // Get status
    #[wasm_bindgen(getter)]
    pub async fn status(&self) -> JsRelayStatus {
        self.inner.status().await.into()
    }

    /// Get `RelayInformationDocument`
    #[wasm_bindgen(getter)]
    pub async fn document(&self) -> JsRelayInformationDocument {
        self.inner.document().await.into()
    }
}
