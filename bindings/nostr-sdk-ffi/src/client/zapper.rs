// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_ffi::nips::nip57::ZapType;
use nostr_ffi::{EventId, PublicKey};
use nostr_sdk::zapper::{DynNostrZapper, IntoNostrZapper};
use nostr_sdk::{client, nwc};
use uniffi::Object;

use crate::nwc::NWC;

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
    pub fn event(event_id: Arc<EventId>) -> Self {
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
    pub fn message(self: Arc<Self>, message: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.message(message);
        builder
    }
}
