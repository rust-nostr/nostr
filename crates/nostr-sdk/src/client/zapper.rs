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

use super::options::SUPPORT_RUST_NOSTR_LUD16;
use super::{Client, Error};
use crate::FilterOptions;

const SUPPORT_RUST_NOSTR_MSG: &str = "Zap split to support Rust Nostr development!";

/// Zap entity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl ClientZapper {
    /// Create a new [WebLN] instance and compose [ClientZapper]
    #[cfg(all(feature = "webln", target_arch = "wasm32"))]
    pub fn webln() -> Result<Self, Error> {
        let instance = WebLN::new()?;
        Ok(Self::WebLN(instance))
    }
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
    ///
    /// This method automatically create a split zap to support Rust Nostr development.
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
        let (public_key, metadata): (XOnlyPublicKey, Metadata) = match to {
            ZapEntity::Event(event_id) => {
                // Get event
                let filter: Filter = Filter::new().id(event_id);
                let events: Vec<Event> = self.get_events_of(vec![filter], None).await?;
                let event: &Event = events.first().ok_or(Error::EventNotFound(event_id))?;
                let public_key: XOnlyPublicKey = event.author();
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
        let _invoices: Vec<String> = self
            .zap_split(public_key, lud, satoshi, details, to.event_id())
            .await?;

        match zapper {
            #[cfg(all(feature = "webln", target_arch = "wasm32"))]
            ClientZapper::WebLN(webln) => {
                webln.enable().await?;
                for invoice in _invoices.into_iter() {
                    webln.send_payment(invoice).await?;
                }
                Ok(())
            }
            #[cfg(feature = "nip47")]
            ClientZapper::NWC(uri) => {
                // Add relay and connect if not exists
                if self.add_relay(uri.relay_url.clone()).await? {
                    self.connect_relay(uri.relay_url.clone()).await?;
                }

                for invoice in _invoices.into_iter() {
                    // Compose NWC request event
                    let req = Request {
                        method: Method::PayInvoice,
                        params: RequestParams::PayInvoice(PayInvoiceRequestParams {
                            id: None,
                            invoice,
                            amount: None,
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
                    match relay
                        .get_events_of(
                            vec![filter],
                            Duration::from_secs(10),
                            FilterOptions::ExitOnEOSE,
                        )
                        .await
                    {
                        Ok(events) => match events.first() {
                            Some(event) => {
                                let decrypt_res =
                                    nip04::decrypt(&uri.secret, &uri.public_key, &event.content)?;
                                let nip47_res = Response::from_json(decrypt_res)?;
                                if let Some(ResponseResult::PayInvoice(pay_invoice_result)) =
                                    nip47_res.result
                                {
                                    tracing::info!(
                                        "Zap sent! Preimage: {}",
                                        pay_invoice_result.preimage
                                    );
                                } else {
                                    tracing::warn!(
                                        "Unexpected NIP47 result: {}",
                                        nip47_res.as_json()
                                    );
                                }
                            }
                            None => {
                                tracing::warn!(
                                    "Zap [apparently] sent (`PayInvoice` response not received)."
                                );
                            }
                        },
                        Err(e) => {
                            tracing::error!("Impossible to get NWC response event: {e}");
                        }
                    }
                }

                Ok(())
            }
        }
    }

    /// Split zap to support Rust Nostr development
    async fn zap_split(
        &self,
        public_key: XOnlyPublicKey,
        lud: Lud06OrLud16,
        satoshi: u64,
        details: Option<ZapDetails>,
        event_id: Option<EventId>,
    ) -> Result<Vec<String>, Error> {
        let mut invoices: Vec<String> = Vec::with_capacity(2);
        let mut msats: u64 = satoshi * 1000;

        // Check if is set a percentage
        if let Some(percentage) = self.opts.get_support_rust_nostr_percentage() {
            let rust_nostr_msats = (satoshi as f64 * percentage * 1000.0) as u64;
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
        };

        // Compose zap request
        let zap_request: Option<String> = match details {
            Some(details) => {
                let mut data = ZapRequestData::new(
                    public_key,
                    [UncheckedUrl::from("wss://nostr.mutinywallet.com")],
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
