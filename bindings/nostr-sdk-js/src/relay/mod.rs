// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod filtering;
pub mod flags;
pub mod limits;
pub mod options;

use self::flags::JsAtomicRelayServiceFlags;
use self::options::JsRelayOptions;
use crate::protocol::event::JsEventId;
use crate::protocol::nips::nip11::JsRelayInformationDocument;

#[derive(Clone)]
#[wasm_bindgen(js_name = Reconciliation)]
pub struct JsReconciliation {
    /// The IDs that were stored locally
    #[wasm_bindgen(getter_with_clone)]
    pub local: Vec<JsEventId>,
    /// The IDs that were missing locally (stored on relay)
    #[wasm_bindgen(getter_with_clone)]
    pub remote: Vec<JsEventId>,
    /// Events that are **successfully** sent to relays during reconciliation
    #[wasm_bindgen(getter_with_clone)]
    pub sent: Vec<JsEventId>,
    /// Event that are **successfully** received from relay
    #[wasm_bindgen(getter_with_clone)]
    pub received: Vec<JsEventId>,
    // TODO: add send_failures:
}

impl From<Reconciliation> for JsReconciliation {
    fn from(value: Reconciliation) -> Self {
        Self {
            local: value.local.into_iter().map(|e| e.into()).collect(),
            remote: value.remote.into_iter().map(|e| e.into()).collect(),
            sent: value.sent.into_iter().map(|e| e.into()).collect(),
            received: value.received.into_iter().map(|e| e.into()).collect(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    /// Array
    #[wasm_bindgen(typescript_type = "JsRelay[]")]
    pub type JsRelayArray;
}

#[wasm_bindgen(js_name = Relay)]
pub struct JsRelay {
    inner: Relay,
}

impl From<Relay> for JsRelay {
    fn from(inner: Relay) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_name = RelayStatus)]
pub enum JsRelayStatus {
    /// Initialized
    Initialized,
    /// Pending
    Pending,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Disconnected, will retry to connect again
    Disconnected,
    /// Completely disconnected
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

    /// Get status
    pub fn status(&self) -> JsRelayStatus {
        self.inner.status().into()
    }

    /// Get Relay Service Flags
    pub fn flags(&self) -> JsAtomicRelayServiceFlags {
        self.inner.flags().clone().into()
    }

    /// Check if relay is connected
    #[wasm_bindgen(js_name = isConnected)]
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Get `RelayInformationDocument`
    pub async fn document(&self) -> JsRelayInformationDocument {
        self.inner.document().await.into()
    }

    /// Get options
    pub fn opts(&self) -> JsRelayOptions {
        self.inner.opts().clone().into()
    }

    // TODO: add stats
}
