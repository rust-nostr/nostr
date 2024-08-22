// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use lnurl_pay::api::Lud06OrLud16;
use lnurl_pay::{LightningAddress, LnUrl};
use nostr::prelude::*;

use super::{Client, Error, EventSource};

/// Zap entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZapEntity {
    /// Zap to event
    Event(EventId),
    /// Zap to public key
    PublicKey(PublicKey),
}

impl From<EventId> for ZapEntity {
    fn from(value: EventId) -> Self {
        Self::Event(value)
    }
}

impl From<Nip19Event> for ZapEntity {
    fn from(value: Nip19Event) -> Self {
        Self::Event(value.event_id)
    }
}

impl From<PublicKey> for ZapEntity {
    fn from(value: PublicKey) -> Self {
        Self::PublicKey(value)
    }
}

impl ZapEntity {
    fn event_id(&self) -> Option<EventId> {
        match self {
            Self::Event(id) => Some(*id),
            _ => None,
        }
    }
}

/// Zap Details
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZapDetails {
    r#type: ZapType,
    message: String,
}

impl ZapDetails {
    /// Create new Zap Details
    ///
    /// **Note: `private` zaps are not currently supported here!**
    pub fn new(zap_type: ZapType) -> Self {
        Self {
            r#type: zap_type,
            message: String::new(),
        }
    }

    /// Add message
    pub fn message<S>(mut self, message: S) -> Self
    where
        S: Into<String>,
    {
        self.message = message.into();
        self
    }
}

impl Client {
    /// Steps
    /// 1. Check if zapper is set and available
    /// 2. Get metadata of pubkey/author of event
    /// 3. Get invoice
    /// 4. Send payment
    pub(super) async fn internal_zap<T>(
        &self,
        to: T,
        satoshi: u64,
        details: Option<ZapDetails>,
    ) -> Result<(), Error>
    where
        T: Into<ZapEntity>,
    {
        // Check if zapper is set
        if !self.has_zapper().await {
            return Err(Error::ZapperNotConfigured);
        }

        // Get entity metadata
        let to: ZapEntity = to.into();
        let (public_key, metadata): (PublicKey, Metadata) = match to {
            ZapEntity::Event(event_id) => {
                // Get event
                let filter: Filter = Filter::new().id(event_id);
                let events: Vec<Event> = self
                    .get_events_of(vec![filter], EventSource::both(Some(self.opts.timeout)))
                    .await?;
                let event: &Event = events.first().ok_or(Error::EventNotFound(event_id))?;
                let public_key: PublicKey = event.pubkey;
                let metadata: Metadata = self.fetch_metadata(public_key, None).await?;
                (public_key, metadata)
            }
            ZapEntity::PublicKey(public_key) => {
                let metadata: Metadata = self.fetch_metadata(public_key, None).await?;
                (public_key, metadata)
            }
        };

        // Parse lud
        let lud: Lud06OrLud16 = if let Some(lud16) = &metadata.lud16 {
            LightningAddress::parse(lud16)?.into()
        } else if let Some(lud06) = &metadata.lud06 {
            LnUrl::from_str(lud06)?.into()
        } else {
            return Err(Error::ImpossibleToZap(String::from("LUD06/LUD16 not set")));
        };

        // Compose zap split and get invoices
        let invoice: String = self
            .compose_zap(public_key, lud, satoshi, details, to.event_id())
            .await?;

        let zapper = self.zapper().await?;
        zapper.pay(invoice).await?;

        Ok(())
    }

    /// Compose zap and get invoice
    async fn compose_zap(
        &self,
        public_key: PublicKey,
        lud: Lud06OrLud16,
        satoshi: u64,
        details: Option<ZapDetails>,
        event_id: Option<EventId>,
    ) -> Result<String, Error> {
        let msats: u64 = satoshi * 1000;

        // Compose zap request
        let zap_request: Option<String> = match details {
            Some(details) => {
                let mut data = ZapRequestData::new(
                    public_key,
                    [
                        UncheckedUrl::from("wss://nostr.mutinywallet.com"),
                        UncheckedUrl::from("wss://relay.mutinywallet.com"),
                    ],
                )
                .amount(msats)
                .message(details.message);
                data.event_id = event_id;
                match details.r#type {
                    ZapType::Public => {
                        let builder = EventBuilder::public_zap_request(data);
                        Some(self.sign_event_builder(builder).await?.as_json())
                    }
                    ZapType::Private => None,
                    ZapType::Anonymous => Some(nip57::anonymous_zap_request(data)?.as_json()),
                }
            }
            None => None,
        };

        // Get invoice
        let invoice: String =
            lnurl_pay::api::get_invoice(lud, msats, None, zap_request, None).await?;

        Ok(invoice)
    }
}
