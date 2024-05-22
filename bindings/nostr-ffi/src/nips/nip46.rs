// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip46::{self, Method, Request, ResponseResult};
use nostr::{JsonUtil, Url};
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::NostrError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct NostrConnectMetadata {
    inner: nip46::NostrConnectMetadata,
}

impl Deref for NostrConnectMetadata {
    type Target = nip46::NostrConnectMetadata;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl NostrConnectMetadata {
    /// New Nostr Connect Metadata
    #[uniffi::constructor]
    pub fn new(name: String) -> Self {
        Self {
            inner: nip46::NostrConnectMetadata::new(name),
        }
    }

    /// URL of the website requesting the connection
    pub fn url(self: Arc<Self>, url: String) -> Result<Self> {
        let url: Url = Url::parse(&url)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.url(url);
        Ok(builder)
    }

    /// Description of the `App`
    pub fn description(self: Arc<Self>, description: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.description(description);
        builder
    }

    /// List of URLs for icons of the `App`
    pub fn icons(self: Arc<Self>, icons: Vec<String>) -> Self {
        let icons: Vec<Url> = icons
            .into_iter()
            .filter_map(|u| Url::parse(&u).ok())
            .collect();
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.icons(icons);
        builder
    }

    /// Serialize as JSON string
    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
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

#[uniffi::export]
impl NostrConnectURI {
    #[uniffi::constructor]
    pub fn parse(uri: &str) -> Result<Self> {
        Ok(Self {
            inner: nip46::NostrConnectURI::parse(uri)?,
        })
    }

    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}

#[derive(Enum)]
pub enum NostrConnectMessage {
    Request {
        id: String,
        method: String,
        params: Vec<String>,
    },
    Response {
        id: String,
        result: Option<String>,
        error: Option<String>,
    },
}

impl TryFrom<NostrConnectMessage> for nip46::Message {
    type Error = NostrError;

    fn try_from(value: NostrConnectMessage) -> Result<Self, Self::Error> {
        Ok(match value {
            NostrConnectMessage::Request { id, method, params } => {
                let method: Method = Method::from_str(&method)?;
                Self::Request {
                    id,
                    req: Request::from_message(method, params)?,
                }
            }
            NostrConnectMessage::Response { id, result, error } => Self::Response {
                id,
                result: match result {
                    Some(a) => Some(ResponseResult::parse(&a)?),
                    None => None,
                },
                error,
            },
        })
    }
}
