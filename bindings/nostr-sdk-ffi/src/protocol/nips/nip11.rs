// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::sync::Arc;

use nostr::nips::nip11;
use nostr::Url;
use uniffi::{Enum, Object, Record};

use crate::error::Result;
use crate::protocol::types::Timestamp;

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct RelayInformationDocument {
    inner: nip11::RelayInformationDocument,
}

impl From<nip11::RelayInformationDocument> for RelayInformationDocument {
    fn from(inner: nip11::RelayInformationDocument) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio", default(proxy = None))]
pub async fn nip11_get_information_document(
    url: &str,
    proxy: Option<String>,
) -> Result<RelayInformationDocument> {
    let url: Url = Url::parse(url)?;
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(RelayInformationDocument {
        inner: nip11::RelayInformationDocument::get(url, proxy).await?,
    })
}

#[uniffi::export]
impl RelayInformationDocument {
    #[uniffi::constructor]
    /// Create new empty [`RelayInformationDocument`]
    pub fn new() -> Self {
        Self {
            inner: nip11::RelayInformationDocument::new(),
        }
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

    pub fn limitation(&self) -> Option<Limitation> {
        self.inner.limitation.clone().map(|l| l.into())
    }

    pub fn retention(&self) -> Vec<Retention> {
        self.inner
            .retention
            .clone()
            .into_iter()
            .map(|l| l.into())
            .collect()
    }

    pub fn relay_countries(&self) -> Vec<String> {
        self.inner.relay_countries.clone()
    }

    pub fn language_tags(&self) -> Vec<String> {
        self.inner.language_tags.clone()
    }

    pub fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    pub fn posting_policy(&self) -> Option<String> {
        self.inner.posting_policy.clone()
    }

    pub fn payments_url(&self) -> Option<String> {
        self.inner.payments_url.clone()
    }

    pub fn fees(&self) -> Option<FeeSchedules> {
        self.inner.fees.clone().map(|f| f.into())
    }

    pub fn icon(&self) -> Option<String> {
        self.inner.icon.clone()
    }
}

/// These are limitations imposed by the relay on clients. Your client should
/// expect that requests which exceed these practical limitations are rejected or fail immediately.
#[derive(Record)]
pub struct Limitation {
    /// Maximum number of bytes for incoming JSON that the relay will attempt to decode and act upon
    pub max_message_length: Option<i32>,
    /// Total number of subscriptions that may be active on a single websocket connection
    pub max_subscriptions: Option<i32>,
    /// Maximum number of filter values in each subscription
    pub max_filters: Option<i32>,
    /// Relay will clamp each filter's limit value to this number
    pub max_limit: Option<i32>,
    /// Maximum length of subscription id as a string
    pub max_subid_length: Option<i32>,
    /// Maximum number of elements in the tags list
    pub max_event_tags: Option<i32>,
    /// Maximum number of characters in the content field of any event
    pub max_content_length: Option<i32>,
    /// New events will require at least this difficulty of PoW,
    pub min_pow_difficulty: Option<i32>,
    /// Relay requires NIP42 authentication to happen before a new connection may perform any other action
    pub auth_required: Option<bool>,
    /// Relay requires payment before a new connection may perform any action
    pub payment_required: Option<bool>,
    /// 'created_at' lower limit
    pub created_at_lower_limit: Option<Arc<Timestamp>>,
    /// 'created_at' upper limit
    pub created_at_upper_limit: Option<Arc<Timestamp>>,
}

impl From<nip11::Limitation> for Limitation {
    fn from(inner: nip11::Limitation) -> Self {
        let nip11::Limitation {
            max_message_length,
            max_subscriptions,
            max_filters,
            max_limit,
            max_subid_length,
            max_event_tags,
            max_content_length,
            min_pow_difficulty,
            auth_required,
            payment_required,
            created_at_lower_limit,
            created_at_upper_limit,
        } = inner;
        Self {
            max_message_length,
            max_subscriptions,
            max_filters,
            max_limit,
            max_subid_length,
            max_event_tags,
            max_content_length,
            min_pow_difficulty,
            auth_required,
            payment_required,
            created_at_lower_limit: created_at_lower_limit.map(|c| Arc::new(c.into())),
            created_at_upper_limit: created_at_upper_limit.map(|c| Arc::new(c.into())),
        }
    }
}

/// A retention schedule for the relay
#[derive(Record)]
pub struct Retention {
    /// The event kinds this retention pertains to
    pub kinds: Option<Vec<RetentionKind>>,
    /// The amount of time these events are kept
    pub time: Option<u64>,
    /// The max number of events kept before removing older events
    pub count: Option<u64>,
}

impl From<nip11::Retention> for Retention {
    fn from(inner: nip11::Retention) -> Self {
        let nip11::Retention { kinds, time, count } = inner;
        Self {
            kinds: kinds.map(|k| k.into_iter().map(|k| k.into()).collect()),
            time,
            count,
        }
    }
}

#[derive(Enum)]
pub enum RetentionKind {
    Single { single: u64 },
    Range { start: u64, end: u64 },
}

impl From<nip11::RetentionKind> for RetentionKind {
    fn from(value: nip11::RetentionKind) -> Self {
        match value {
            nip11::RetentionKind::Single(s) => Self::Single { single: s },
            nip11::RetentionKind::Range(s, e) => Self::Range { start: s, end: e },
        }
    }
}

/// Available fee schedules
#[derive(Record)]
pub struct FeeSchedules {
    /// Fees for admission to use the relay
    pub admission: Vec<FeeSchedule>,
    /// Fees for subscription to use the relay
    pub subscription: Vec<FeeSchedule>,
    /// Fees to publish to the relay
    pub publication: Vec<FeeSchedule>,
}

impl From<nip11::FeeSchedules> for FeeSchedules {
    fn from(inner: nip11::FeeSchedules) -> Self {
        let nip11::FeeSchedules {
            admission,
            subscription,
            publication,
        } = inner;
        Self {
            admission: admission.into_iter().map(|a| a.into()).collect(),
            subscription: subscription.into_iter().map(|s| s.into()).collect(),
            publication: publication.into_iter().map(|p| p.into()).collect(),
        }
    }
}

/// The specific information about a fee schedule
#[derive(Record)]
pub struct FeeSchedule {
    /// The fee amount
    pub amount: i32,
    /// The denomination of the feed
    pub unit: String,
    /// The duration for which the fee is valid
    pub period: Option<i32>,
    /// The event kinds the fee allows the client to publish to the relay
    pub kinds: Option<Vec<String>>,
}

impl From<nip11::FeeSchedule> for FeeSchedule {
    fn from(inner: nip11::FeeSchedule) -> Self {
        let nip11::FeeSchedule {
            amount,
            unit,
            period,
            kinds,
        } = inner;
        Self {
            amount,
            unit,
            period,
            kinds,
        }
    }
}
