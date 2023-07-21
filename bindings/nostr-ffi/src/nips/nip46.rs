// Copyright (c) 2022-2023 Coinstr
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::nips::nip46;

use crate::error::Result;

pub struct NostrConnectURI {
    inner: nip46::NostrConnectURI,
}

impl From<nip46::NostrConnectURI> for NostrConnectURI {
    fn from(inner: nip46::NostrConnectURI) -> Self {
        Self { inner }
    }
}

impl Deref for NostrConnectURI {
    type Target = nip46::NostrConnectURI;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NostrConnectURI {
    pub fn from_string(uri: String) -> Result<Self> {
        Ok(Self {
            inner: nip46::NostrConnectURI::from_str(&uri)?,
        })
    }

    pub fn public_key(&self) -> String {
        self.inner.public_key.to_string()
    }

    pub fn relay_url(&self) -> String {
        self.inner.relay_url.to_string()
    }

    pub fn name(&self) -> String {
        self.inner.metadata.name.clone()
    }

    pub fn url(&self) -> Option<String> {
        self.inner.metadata.url.as_ref().map(|u| u.to_string())
    }

    pub fn description(&self) -> Option<String> {
        self.inner.metadata.description.clone()
    }
}
