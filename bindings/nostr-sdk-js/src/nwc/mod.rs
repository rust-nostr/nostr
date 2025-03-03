// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nwc::prelude::*;
use wasm_bindgen::prelude::*;

pub mod options;

use self::options::JsNostrWalletConnectOptions;
use crate::error::{into_err, Result};
use crate::protocol::nips::nip47::{
    JsGetInfoResponse, JsListTransactionsRequest, JsLookupInvoiceRequest, JsLookupInvoiceResponse,
    JsMakeInvoiceRequest, JsMakeInvoiceResponse, JsNostrWalletConnectURI, JsPayInvoiceRequest,
    JsPayInvoiceResponse, JsPayKeysendRequest, JsPayKeysendResponse,
};

#[wasm_bindgen]
extern "C" {
    /// Array
    #[wasm_bindgen(typescript_type = "LookupInvoiceResponse[]")]
    pub type JsLookupInvoiceResponseArray;
}

/// Nostr Wallet Connect client
#[wasm_bindgen(js_name = NWC)]
pub struct JsNwc {
    inner: NWC,
}

impl Deref for JsNwc {
    type Target = NWC;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = NWC)]
impl JsNwc {
    /// Compose new `NWC` client
    #[wasm_bindgen(constructor)]
    pub fn new(uri: &JsNostrWalletConnectURI) -> Self {
        Self {
            inner: NWC::new(uri.deref().clone()),
        }
    }

    /// Compose new `NWC` client with `NostrWalletConnectOptions`
    #[wasm_bindgen(js_name = withOpts)]
    pub fn with_opts(uri: &JsNostrWalletConnectURI, opts: &JsNostrWalletConnectOptions) -> Self {
        Self {
            inner: NWC::with_opts(uri.deref().clone(), opts.deref().clone()),
        }
    }

    /// Pay invoice
    #[wasm_bindgen(js_name = payInvoice)]
    pub async fn pay_invoice(&self, params: &JsPayInvoiceRequest) -> Result<JsPayInvoiceResponse> {
        Ok(self
            .inner
            .pay_invoice(params.to_owned().into())
            .await
            .map_err(into_err)?
            .into())
    }

    /// Pay keysend
    #[wasm_bindgen(js_name = payKeysend)]
    pub async fn pay_keysend(&self, params: &JsPayKeysendRequest) -> Result<JsPayKeysendResponse> {
        Ok(self
            .inner
            .pay_keysend(params.to_owned().into())
            .await
            .map_err(into_err)?
            .into())
    }

    /// Create invoice
    #[wasm_bindgen(js_name = makeInvoice)]
    pub async fn make_invoice(
        &self,
        params: &JsMakeInvoiceRequest,
    ) -> Result<JsMakeInvoiceResponse> {
        Ok(self
            .inner
            .make_invoice(params.to_owned().into())
            .await
            .map_err(into_err)?
            .into())
    }

    /// Lookup invoice
    #[wasm_bindgen(js_name = lookupInvoice)]
    pub async fn lookup_invoice(
        &self,
        params: &JsLookupInvoiceRequest,
    ) -> Result<JsLookupInvoiceResponse> {
        Ok(self
            .inner
            .lookup_invoice(params.to_owned().into())
            .await
            .map_err(into_err)?
            .into())
    }

    /// List transactions
    #[wasm_bindgen(js_name = listTransactions)]
    pub async fn list_transactions(
        &self,
        params: &JsListTransactionsRequest,
    ) -> Result<JsLookupInvoiceResponseArray> {
        let list = self
            .inner
            .list_transactions(params.to_owned().into())
            .await
            .map_err(into_err)?;
        Ok(list
            .into_iter()
            .map(|e| {
                let e: JsLookupInvoiceResponse = e.into();
                JsValue::from(e)
            })
            .collect::<Array>()
            .unchecked_into())
    }

    /// Get balance
    #[wasm_bindgen(js_name = getBalance)]
    pub async fn get_balance(&self) -> Result<u64> {
        self.inner.get_balance().await.map_err(into_err)
    }

    /// Get info
    #[wasm_bindgen(js_name = getInfo)]
    pub async fn get_info(&self) -> Result<JsGetInfoResponse> {
        Ok(self.inner.get_info().await.map_err(into_err)?.into())
    }
}
