// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::sync::Arc;

use nostr::nips::nip11;
use nostr::Url;
use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct RelayInformationDocument {
    inner: nip11::RelayInformationDocument,
}

impl From<nip11::RelayInformationDocument> for RelayInformationDocument {
    fn from(inner: nip11::RelayInformationDocument) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl RelayInformationDocument {
    #[uniffi::constructor]
    pub fn get(url: String, proxy: Option<String>) -> Result<Arc<Self>> {
        let url: Url = Url::parse(&url)?;
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse()?),
            None => None,
        };
        Ok(Arc::new(Self {
            inner: nip11::RelayInformationDocument::get_blocking(url, proxy)?,
        }))
    }

    pub fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    pub fn pubkey(&self) -> Option<String> {
        self.inner.pubkey.clone()
    }

    pub fn contact(&self) -> Option<String> {
        self.inner.contact.clone()
    }

    pub fn supported_nips(&self) -> Option<Vec<u16>> {
        self.inner.supported_nips.clone()
    }

    pub fn software(&self) -> Option<String> {
        self.inner.software.clone()
    }

    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }
}
