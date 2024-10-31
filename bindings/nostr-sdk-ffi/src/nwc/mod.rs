// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::nwc;
use uniffi::Object;

pub mod options;

use self::options::NostrWalletConnectOptions;
use crate::error::Result;
use crate::protocol::nips::nip47::{
    GetInfoResponseResult, ListTransactionsRequestParams, LookupInvoiceRequestParams,
    LookupInvoiceResponseResult, MakeInvoiceRequestParams, MakeInvoiceResponseResult,
    NostrWalletConnectURI, PayKeysendRequestParams, PayKeysendResponseResult,
};

/// Nostr Wallet Connect client
#[derive(Object)]
pub struct NWC {
    inner: nwc::NWC,
}

impl Deref for NWC {
    type Target = nwc::NWC;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export(async_runtime = "tokio")]
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
    pub async fn pay_invoice(&self, invoice: String) -> Result<String> {
        Ok(self.inner.pay_invoice(invoice).await?)
    }

    /// Pay keysend
    pub async fn pay_keysend(
        &self,
        params: PayKeysendRequestParams,
    ) -> Result<PayKeysendResponseResult> {
        Ok(self.inner.pay_keysend(params.into()).await?.into())
    }

    /// Create invoice
    pub async fn make_invoice(
        &self,
        params: MakeInvoiceRequestParams,
    ) -> Result<MakeInvoiceResponseResult> {
        Ok(self.inner.make_invoice(params.into()).await?.into())
    }

    /// Lookup invoice
    pub async fn lookup_invoice(
        &self,
        params: LookupInvoiceRequestParams,
    ) -> Result<LookupInvoiceResponseResult> {
        Ok(self.inner.lookup_invoice(params.into()).await?.into())
    }

    /// List transactions
    pub async fn list_transactions(
        &self,
        params: ListTransactionsRequestParams,
    ) -> Result<Vec<LookupInvoiceResponseResult>> {
        let list = self.inner.list_transactions(params.into()).await?;
        Ok(list.into_iter().map(|l| l.into()).collect())
    }

    /// Get balance
    pub async fn get_balance(&self) -> Result<u64> {
        Ok(self.inner.get_balance().await?)
    }

    /// Get info
    pub async fn get_info(&self) -> Result<GetInfoResponseResult> {
        Ok(self.inner.get_info().await?.into())
    }
}
