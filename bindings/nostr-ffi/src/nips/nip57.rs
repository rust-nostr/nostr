// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip57;
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::{Event, EventId, Keys, PublicKey, SecretKey};

#[derive(Enum)]
pub enum ZapType {
    /// Public
    Public,
    /// Private
    Private,
    /// Anonymous
    Anonymous,
}

impl From<ZapType> for nip57::ZapType {
    fn from(value: ZapType) -> Self {
        match value {
            ZapType::Public => Self::Public,
            ZapType::Private => Self::Private,
            ZapType::Anonymous => Self::Anonymous,
        }
    }
}

#[derive(Clone, Object)]
pub struct ZapRequestData {
    inner: nip57::ZapRequestData,
}

impl Deref for ZapRequestData {
    type Target = nip57::ZapRequestData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nip57::ZapRequestData> for ZapRequestData {
    fn from(inner: nip57::ZapRequestData) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl ZapRequestData {
    #[uniffi::constructor]
    pub fn new(public_key: Arc<PublicKey>, relays: Vec<String>) -> Self {
        Self {
            inner: nip57::ZapRequestData::new(
                public_key.as_ref().into(),
                relays.into_iter().map(|r| r.into()),
            ),
        }
    }

    pub fn message(self: Arc<Self>, message: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.message(message);
        Arc::new(builder)
    }

    pub fn amount(self: Arc<Self>, amount: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.amount(amount);
        Arc::new(builder)
    }

    pub fn lnurl(self: Arc<Self>, lnurl: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.lnurl(lnurl);
        Arc::new(builder)
    }

    pub fn event_id(self: Arc<Self>, event_id: Arc<EventId>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.event_id(event_id.as_ref().into());
        Arc::new(builder)
    }
}

#[uniffi::export]
pub fn nip57_anonymous_zap_request(data: Arc<ZapRequestData>) -> Result<Event> {
    Ok(nip57::anonymous_zap_request(data.as_ref().deref().clone())?.into())
}

#[uniffi::export]
pub fn nip57_private_zap_request(data: Arc<ZapRequestData>, keys: Arc<Keys>) -> Result<Event> {
    Ok(nip57::private_zap_request(data.as_ref().deref().clone(), keys.deref())?.into())
}

#[uniffi::export]
pub fn decrypt_sent_private_zap_message(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
    private_zap: Arc<Event>,
) -> Result<Event> {
    Ok(nip57::decrypt_sent_private_zap_message(
        secret_key.deref(),
        public_key.deref(),
        private_zap.deref(),
    )?
    .into())
}

#[uniffi::export]
pub fn decrypt_received_private_zap_message(
    secret_key: Arc<SecretKey>,
    private_zap: Arc<Event>,
) -> Result<Event> {
    Ok(
        nip57::decrypt_received_private_zap_message(secret_key.deref(), private_zap.deref())?
            .into(),
    )
}
