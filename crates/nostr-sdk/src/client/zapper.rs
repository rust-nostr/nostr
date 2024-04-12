// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use lnurl_pay::api::Lud06OrLud16;
use lnurl_pay::{LightningAddress, LnUrl};
use nostr::prelude::*;

use super::{Client, Error};

const SUPPORT_RUST_NOSTR_LUD16: &str = "yuki@getalby.com"; // TODO: use a rust-nostr dedicated LUD16
const SUPPORT_RUST_NOSTR_PERCENTAGE: f64 = 0.05; // 5%
const SUPPORT_RUST_NOSTR_MSG: &str = "Zap split to support Rust Nostr development!";

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
    /// 1. Check if zapper is set and availabe
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
                let events: Vec<Event> = self.get_events_of(vec![filter], None).await?;
                let event: &Event = events.first().ok_or(Error::EventNotFound(event_id))?;
                let public_key: PublicKey = event.author();
                let metadata: Metadata = self.metadata(public_key).await?;
                (public_key, metadata)
            }
            ZapEntity::PublicKey(public_key) => {
                let metadata: Metadata = self.metadata(public_key).await?;
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
        let invoices: Vec<String> = self
            .zap_split(public_key, lud, satoshi, details, to.event_id())
            .await?;

        let zapper = self.zapper().await?;
        for invoice in invoices.into_iter() {
            zapper.pay(invoice).await?;
        }
        Ok(())
    }

    /// Split zap to support Rust Nostr development
    async fn zap_split(
        &self,
        public_key: PublicKey,
        lud: Lud06OrLud16,
        satoshi: u64,
        details: Option<ZapDetails>,
        event_id: Option<EventId>,
    ) -> Result<Vec<String>, Error> {
        let mut invoices: Vec<String> = Vec::with_capacity(2);
        let mut msats: u64 = satoshi * 1000;

        let rust_nostr_sats: u64 = (satoshi as f64 * SUPPORT_RUST_NOSTR_PERCENTAGE) as u64;
        let rust_nostr_msats: u64 = rust_nostr_sats * 1000;
        let rust_nostr_lud = LightningAddress::parse(SUPPORT_RUST_NOSTR_LUD16)?;
        let rust_nostr_lud = Lud06OrLud16::Lud16(rust_nostr_lud);

        // Check if LUD is equal to Rust Nostr LUD
        if rust_nostr_lud != lud {
            match lnurl_pay::api::get_invoice(
                rust_nostr_lud,
                rust_nostr_msats,
                Some(SUPPORT_RUST_NOSTR_MSG.to_string()),
                None,
                None,
            )
            .await
            {
                Ok(invoice) => {
                    invoices.push(invoice);
                    msats = satoshi * 1000 - rust_nostr_msats;
                }
                Err(e) => {
                    tracing::error!("Impossible to get invoice for Rust Nostr: {e}");
                }
            }
        }

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
        invoices.push(invoice);

        Ok(invoices)
    }
}
