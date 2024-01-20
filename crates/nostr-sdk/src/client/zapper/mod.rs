// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Zapper

use std::str::FromStr;
#[cfg(feature = "nip47")]
use std::time::Duration;

use lnurl_pay::api::Lud06OrLud16;
use lnurl_pay::{LightningAddress, LnUrl};
#[cfg(feature = "nip47")]
use nostr::nips::nip04;
use nostr::nips::nip19::Nip19Event;
#[cfg(feature = "nip47")]
use nostr::nips::nip47::{
    Method, NostrWalletConnectURI, PayInvoiceRequestParams, Request, RequestParams, Response,
    ResponseResult,
};
use nostr::nips::nip57::{self, ZapRequestData, ZapType};
use nostr::secp256k1::XOnlyPublicKey;
#[cfg(feature = "nip47")]
use nostr::Kind;
use nostr::{Event, EventBuilder, EventId, Filter, JsonUtil, Metadata, UncheckedUrl};
#[cfg(all(feature = "webln", target_arch = "wasm32"))]
use webln::WebLN;

use super::{Client, Error};
use crate::FilterOptions;

/// Zap entity
pub enum ZapEntity {
    /// Zap to event
    Event(EventId),
    /// Zap to public key
    PublicKey(XOnlyPublicKey),
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

impl From<XOnlyPublicKey> for ZapEntity {
    fn from(value: XOnlyPublicKey) -> Self {
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

/// Client Zapper
#[derive(Debug, Clone)]
pub enum ClientZapper {
    /// WebLN
    #[cfg(all(feature = "webln", target_arch = "wasm32"))]
    WebLN(WebLN),
    /// NWC
    #[cfg(feature = "nip47")]
    NWC(NostrWalletConnectURI),
}

#[cfg(all(feature = "webln", target_arch = "wasm32"))]
impl From<WebLN> for ClientZapper {
    fn from(value: WebLN) -> Self {
        Self::WebLN(value)
    }
}

#[cfg(feature = "nip47")]
impl From<NostrWalletConnectURI> for ClientZapper {
    fn from(value: NostrWalletConnectURI) -> Self {
        Self::NWC(value)
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
    /// Send a Zap!
    pub async fn zap<T>(
        &self,
        to: T,
        satoshi: u64,
        details: Option<ZapDetails>,
    ) -> Result<(), Error>
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
        let (public_key, lud): (XOnlyPublicKey, Lud06OrLud16) = match to {
            ZapEntity::Event(event_id) => {
                // Get event
                let filter: Filter = Filter::new().id(event_id);
                let events: Vec<Event> = self.get_events_of(vec![filter], None).await?;
                let event: &Event = events.first().ok_or(Error::EventNotFound(event_id))?;
                let public_key: XOnlyPublicKey = event.author();
                let metadata: Metadata = self.metadata(public_key).await?;

                if let Some(lud16) = &metadata.lud16 {
                    (public_key, LightningAddress::parse(lud16)?.into())
                } else if let Some(lud06) = &metadata.lud06 {
                    (public_key, LnUrl::from_str(lud06)?.into())
                } else {
                    return Err(Error::ImpossibleToZap(String::from("LUD06/LUD16 not set")));
                }
            }
            ZapEntity::PublicKey(public_key) => {
                let metadata: Metadata = self.metadata(public_key).await?;

                if let Some(lud16) = &metadata.lud16 {
                    (public_key, LightningAddress::parse(lud16)?.into())
                } else if let Some(lud06) = &metadata.lud06 {
                    (public_key, LnUrl::from_str(lud06)?.into())
                } else {
                    return Err(Error::ImpossibleToZap(String::from("LUD06/LUD16 not set")));
                }
            }
        };

        // Compose zap request
        let zap_request: Option<String> = match details {
            Some(details) => {
                let mut data = ZapRequestData::new(
                    public_key,
                    [UncheckedUrl::from("wss://nostr.mutinywallet.com")],
                )
                .amount(satoshi * 1000)
                .message(details.message);
                data.event_id = to.event_id();
                match details.r#type {
                    ZapType::Public => {
                        let builder = EventBuilder::public_zap_request(data);
                        Some(self.internal_sign_event_builder(builder).await?.as_json())
                    }
                    ZapType::Private => None,
                    ZapType::Anonymous => Some(nip57::anonymous_zap_request(data)?.as_json()),
                }
            }
            None => None,
        };

        // Get invoice
        let _invoice: String =
            lnurl_pay::api::get_invoice(lud, satoshi * 1000, zap_request, None).await?;

        match zapper {
            #[cfg(all(feature = "webln", target_arch = "wasm32"))]
            ClientZapper::WebLN(webln) => {
                webln.enable().await?;
                webln.send_payment(_invoice).await?;
                Ok(())
            }
            #[cfg(feature = "nip47")]
            ClientZapper::NWC(uri) => {
                // Add relay and connect if not exists
                if self.add_relay(uri.relay_url.clone()).await? {
                    self.connect_relay(uri.relay_url.clone()).await?;
                }

                // Compose NWC request event
                let req = Request {
                    method: Method::PayInvoice,
                    params: RequestParams::PayInvoice(PayInvoiceRequestParams {
                        invoice: _invoice,
                    }),
                };
                let event = req.to_event(&uri)?;
                let event_id = event.id;

                // Send request
                self.send_event_to(uri.relay_url.clone(), event).await?;

                // Get response
                let relay = self.relay(uri.relay_url.clone()).await?;
                let filter = Filter::new()
                    .author(uri.public_key)
                    .kind(Kind::WalletConnectResponse)
                    .event(event_id)
                    .limit(1);
                let events = relay
                    .get_events_of(
                        vec![filter],
                        Duration::from_secs(10),
                        FilterOptions::ExitOnEOSE,
                    )
                    .await
                    .map_err(|e| Error::RelayPool(crate::relay::pool::Error::Relay(e)))?;
                match events.first() {
                    Some(event) => {
                        let decrypt_res =
                            nip04::decrypt(&uri.secret, &uri.public_key, &event.content)?;
                        let nip47_res = Response::from_json(decrypt_res)?;
                        if let Some(ResponseResult::PayInvoice(pay_invoice_result)) =
                            nip47_res.result
                        {
                            tracing::info!("Zap sent! Preimage: {}", pay_invoice_result.preimage);
                        } else {
                            tracing::warn!("Unexpected NIP47 result: {}", nip47_res.as_json());
                            return Err(Error::ResponseNotMatchRequest);
                        }
                    }
                    None => {
                        tracing::warn!(
                            "Zap [apparently] sent (`PayInvoice` response not received)."
                        );
                    }
                }

                Ok(())
            }
        }
    }
}