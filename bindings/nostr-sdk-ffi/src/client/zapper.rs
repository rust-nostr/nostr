// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use nostr_sdk::client;
use nostr_sdk::zapper::{DynNostrZapper, IntoNostrZapper};
use uniffi::Object;

use crate::nwc::NWC;
use crate::protocol::event::EventId;
use crate::protocol::key::PublicKey;
use crate::protocol::nips::nip57::ZapType;

/// Zap entity
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct ZapEntity {
    inner: client::ZapEntity,
}

impl Deref for ZapEntity {
    type Target = client::ZapEntity;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl ZapEntity {
    #[uniffi::constructor]
    pub fn event(event_id: &EventId) -> Self {
        Self {
            inner: client::ZapEntity::Event(**event_id),
        }
    }

    #[uniffi::constructor]
    pub fn public_key(public_key: &PublicKey) -> Self {
        Self {
            inner: client::ZapEntity::PublicKey(**public_key),
        }
    }
}

/// Nostr Zapper
#[derive(Object)]
pub struct NostrZapper {
    inner: Arc<DynNostrZapper>,
}

impl Deref for NostrZapper {
    type Target = Arc<DynNostrZapper>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Arc<DynNostrZapper>> for NostrZapper {
    fn from(inner: Arc<DynNostrZapper>) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl NostrZapper {
    #[uniffi::constructor]
    pub fn nwc(client: &NWC) -> Self {
        let zapper: nwc::NWC = client.deref().clone();
        Self {
            inner: zapper.into_nostr_zapper(),
        }
    }
}

/// Zap Details
#[derive(Debug, Clone, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct ZapDetails {
    inner: client::ZapDetails,
}

impl Deref for ZapDetails {
    type Target = client::ZapDetails;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl ZapDetails {
    /// Create new Zap Details
    ///
    /// **Note: `private` zaps are not currently supported here!**
    #[uniffi::constructor]
    pub fn new(zap_type: ZapType) -> Self {
        Self {
            inner: client::ZapDetails::new(zap_type.into()),
        }
    }

    /// Add message
    pub fn message(&self, message: String) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.message(message);
        builder
    }
}
