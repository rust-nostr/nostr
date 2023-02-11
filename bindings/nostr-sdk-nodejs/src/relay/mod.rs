// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use napi::bindgen_prelude::ToNapiValue;
use nostr_nodejs::nips::nip11::JsRelayInformationDocument;
use nostr_sdk::prelude::*;

#[napi(js_name = "Relay")]
pub struct JsRelay {
    inner: Relay,
}

impl From<Relay> for JsRelay {
    fn from(relay: Relay) -> Self {
        Self { inner: relay }
    }
}

#[napi]
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

#[napi]
impl JsRelay {
    /// Get relay url
    #[napi(getter)]
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    /// Get proxy
    #[napi(getter)]
    pub fn proxy(&self) -> Option<String> {
        self.inner.proxy().map(|p| p.to_string())
    }

    // Get status
    #[napi(getter)]
    pub async fn status(&self) -> JsRelayStatus {
        self.inner.status().await.into()
    }

    /// Get `RelayInformationDocument`
    #[napi(getter)]
    pub async fn document(&self) -> JsRelayInformationDocument {
        self.inner.document().await.into()
    }
}
