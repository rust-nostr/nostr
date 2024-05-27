// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_ffi::nips::nip47::{
    GetInfoResponseResult, ListTransactionsRequestParams, LookupInvoiceRequestParams,
    LookupInvoiceResponseResult, MakeInvoiceRequestParams, MakeInvoiceResponseResult,
    NostrWalletConnectURI, PayKeysendRequestParams, PayKeysendResponseResult,
};
use nostr_sdk::{block_on, nwc};
use uniffi::Object;

pub mod options;

use self::options::NostrWalletConnectOptions;
use crate::error::Result;

/// Nostr Wallet Connect client
#[derive(Object)]
pub struct NWC {
    inner: nwc::NWC,
}

#[uniffi::export]
impl NWC {
    /// Compose new `NWC` client
    #[uniffi::constructor]
    pub fn new(uri: &NostrWalletConnectURI) -> Self {
        Self {
            inner: nwc::NWC::new(uri.deref().clone()),
        }
    }

    /// Compose new `NWC` client with `NostrWalletConnectOptions`
    #[uniffi::constructor]
    pub fn with_opts(uri: &NostrWalletConnectURI, opts: &NostrWalletConnectOptions) -> Self {
        Self {
            inner: nwc::NWC::with_opts(uri.deref().clone(), opts.deref().clone()),
        }
    }

    /// Pay invoice
    pub fn pay_invoice(&self, invoice: String) -> Result<String> {
        block_on(async move { Ok(self.inner.pay_invoice(invoice).await?) })
    }

    /// Pay keysend
    pub fn pay_keysend(&self, params: PayKeysendRequestParams) -> Result<PayKeysendResponseResult> {
        block_on(async move { Ok(self.inner.pay_keysend(params.into()).await?.into()) })
    }

    /// Create invoice
    pub fn make_invoice(
        &self,
        params: MakeInvoiceRequestParams,
    ) -> Result<MakeInvoiceResponseResult> {
        block_on(async move { Ok(self.inner.make_invoice(params.into()).await?.into()) })
    }

    /// Lookup invoice
    pub fn lookup_invoice(
        &self,
        params: LookupInvoiceRequestParams,
    ) -> Result<LookupInvoiceResponseResult> {
        block_on(async move { Ok(self.inner.lookup_invoice(params.into()).await?.into()) })
    }

    /// List transactions
    pub fn list_transactions(
        &self,
        params: ListTransactionsRequestParams,
    ) -> Result<Vec<LookupInvoiceResponseResult>> {
        block_on(async move {
            let list = self.inner.list_transactions(params.into()).await?;
            Ok(list.into_iter().map(|l| l.into()).collect())
        })
    }

    /// Get balance
    pub fn get_balance(&self) -> Result<u64> {
        block_on(async move { Ok(self.inner.get_balance().await?) })
    }

    /// Get info
    pub fn get_info(&self) -> Result<GetInfoResponseResult> {
        block_on(async move { Ok(self.inner.get_info().await?.into()) })
    }
}
