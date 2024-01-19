// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Zapper

use std::str::FromStr;

use lnurl_pay::api::Lud06OrLud16;
use lnurl_pay::{LightningAddress, LnUrl};
use nostr::nips::nip57::ZapType;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Filter, Metadata};
#[cfg(all(feature = "webln", target_arch = "wasm32"))]
use webln::WebLN;

use super::{Client, Error};

/// Zap entity
pub enum ZapEntity {
    /// Zap to event
    Event(EventId),
    /// Zap to public key
    PublicKey(XOnlyPublicKey),
    /// Lightning Address
    LUD16(LightningAddress),
    /// LUD06
    LUD06(LnUrl),
}

impl From<EventId> for ZapEntity {
    fn from(value: EventId) -> Self {
        Self::Event(value)
    }
}

impl From<XOnlyPublicKey> for ZapEntity {
    fn from(value: XOnlyPublicKey) -> Self {
        Self::PublicKey(value)
    }
}

/// Client Zapper
#[derive(Debug, Clone)]
pub enum ClientZapper {
    /// WebLN
    #[cfg(all(feature = "webln", target_arch = "wasm32"))]
    WebLN(WebLN),
    /// NWC (TODO)
    NWC,
}

#[cfg(all(feature = "webln", target_arch = "wasm32"))]
impl From<WebLN> for ClientZapper {
    fn from(value: WebLN) -> Self {
        Self::WebLN(value)
    }
}

impl Client {
    /// Send a Zap!
    pub async fn zap<T>(&self, to: T, satoshi: u64, r#_type: Option<ZapType>) -> Result<(), Error>
    where
        T: Into<ZapEntity>,
    {
        // Steps
        // 1. Check if zapper is set and availabe
        // 2. Get metadata of pubkey/author of event
        // 3. Get invoice
        // 4. Send payment

        // Check zapper
        let zapper: ClientZapper = self.zapper().await?;

        // Get entity metadata
        let to: ZapEntity = to.into();
        let lud: Lud06OrLud16 = match to {
            ZapEntity::Event(event_id) => {
                // Get event
                let filter: Filter = Filter::new().id(event_id);
                let events: Vec<Event> = self.get_events_of(vec![filter], None).await?;
                let event: &Event = events.first().ok_or(Error::EventNotFound(event_id))?;
                let public_key: XOnlyPublicKey = event.author();
                let metadata: Metadata = self.metadata(public_key).await?;

                if let Some(lud16) = &metadata.lud16 {
                    LightningAddress::parse(lud16)?.into()
                } else if let Some(lud06) = &metadata.lud06 {
                    LnUrl::from_str(lud06)?.into()
                } else {
                    return Err(Error::ImpossibleToZap(String::from("LUD06/LUD16 not set")));
                }
            }
            ZapEntity::PublicKey(public_key) => {
                let metadata: Metadata = self.metadata(public_key).await?;

                if let Some(lud16) = &metadata.lud16 {
                    LightningAddress::parse(lud16)?.into()
                } else if let Some(lud06) = &metadata.lud06 {
                    LnUrl::from_str(lud06)?.into()
                } else {
                    return Err(Error::ImpossibleToZap(String::from("LUD06/LUD16 not set")));
                }
            }
            ZapEntity::LUD16(lnaddr) => lnaddr.into(),
            ZapEntity::LUD06(lud06) => lud06.into(),
        };

        // Get invoice
        let _invoice: String = lnurl_pay::api::get_invoice(lud, satoshi * 1000, None, None).await?;

        match zapper {
            #[cfg(all(feature = "webln", target_arch = "wasm32"))]
            ClientZapper::WebLN(webln) => {
                webln.enable().await?;
                webln.send_payment(_invoice).await?;
            }
            ClientZapper::NWC => {}
        };

        Ok(())
    }
}
