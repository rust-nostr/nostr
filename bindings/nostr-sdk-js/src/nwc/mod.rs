// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nostr_js::error::{into_err, Result};
use nostr_js::nips::nip47::{
    JsGetInfoResponseResult, JsListTransactionsRequestParams, JsLookupInvoiceRequestParams,
    JsLookupInvoiceResponseResult, JsMakeInvoiceRequestParams, JsMakeInvoiceResponseResult,
    JsNostrWalletConnectURI, JsPayKeysendRequestParams, JsPayKeysendResponseResult,
};
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod options;

use self::options::JsNostrWalletConnectOptions;

#[wasm_bindgen]
extern "C" {
    /// Array
    #[wasm_bindgen(typescript_type = "LookupInvoiceResponseResult[]")]
    pub type JsLookupInvoiceResponseResultArray;
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
    pub async fn pay_invoice(&self, invoice: &str) -> Result<String> {
        self.inner.pay_invoice(invoice).await.map_err(into_err)
    }

    /// Pay keysend
    #[wasm_bindgen(js_name = payKeysend)]
    pub async fn pay_keysend(
        &self,
        params: &JsPayKeysendRequestParams,
    ) -> Result<JsPayKeysendResponseResult> {
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
        params: &JsMakeInvoiceRequestParams,
    ) -> Result<JsMakeInvoiceResponseResult> {
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
        params: &JsLookupInvoiceRequestParams,
    ) -> Result<JsLookupInvoiceResponseResult> {
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
        params: &JsListTransactionsRequestParams,
    ) -> Result<JsLookupInvoiceResponseResultArray> {
        let list = self
            .inner
            .list_transactions(params.to_owned().into())
            .await
            .map_err(into_err)?;
        Ok(list
            .into_iter()
            .map(|e| {
                let e: JsLookupInvoiceResponseResult = e.into();
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
    pub async fn get_info(&self) -> Result<JsGetInfoResponseResult> {
        Ok(self.inner.get_info().await.map_err(into_err)?.into())
    }
}
