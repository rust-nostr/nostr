// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(missing_docs)]

//! Nostr Connect Client (NIP46)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use std::sync::Arc;
use std::time::Duration;

use nostr::nips::nip46::{Message, Request, Response};
use nostr::nips::{nip04, nip46};
use nostr::secp256k1::XOnlyPublicKey;
use nostr::serde_json;
use nostr::{ClientMessage, EventBuilder, Filter, Kind, SubscriptionId, Timestamp, Url};
use tokio::sync::Mutex;

use super::Client;
use crate::relay::RelayPoolNotification;
use crate::time;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Keys(#[from] nostr::key::Error),
    #[error(transparent)]
    Builder(#[from] nostr::event::builder::Error),
    #[error(transparent)]
    Client(#[from] super::Error),
    #[error(transparent)]
    Nip04(#[from] nip04::Error),
    #[error(transparent)]
    Nip46(#[from] nip46::Error),
    #[error(transparent)]
    JSON(#[from] serde_json::Error),
    #[error("generic error")]
    Generic,
    #[error("response error: {0}")]
    Response(String),
    #[error("signer public key not found")]
    SignerPublicKeyNotFound,
    /// Timeout
    #[error("timeout")]
    Timeout,
}

/// Nostr Connect Client Ext
#[derive(Debug, Clone)]
pub(crate) struct NostrConnect {
    relay_url: Url,
    signer_public_key: Arc<Mutex<Option<XOnlyPublicKey>>>,
}

impl NostrConnect {
    pub fn new(relay_url: Url, signer_public_key: Option<XOnlyPublicKey>) -> Self {
        Self {
            relay_url,
            signer_public_key: Arc::new(Mutex::new(signer_public_key)),
        }
    }

    pub fn relay_url(&self) -> Url {
        self.relay_url.clone()
    }

    pub async fn signer_public_key(&self) -> Option<XOnlyPublicKey> {
        let pubkey = self.signer_public_key.lock().await;
        *pubkey
    }

    pub(crate) async fn set_signer_public_key(&self, public_key: XOnlyPublicKey) {
        let mut pubkey = self.signer_public_key.lock().await;
        *pubkey = Some(public_key);
    }
}

impl Client {
    pub async fn init_nostr_connect(&self, timeout: Option<Duration>) -> Result<(), Error> {
        let connect: &NostrConnect = self
            .connect
            .as_ref()
            .ok_or(Error::Client(super::Error::NIP46ClientNotConfigured))?;

        let id = SubscriptionId::generate();
        let filter = Filter::new()
            .pubkey(self.keys.public_key())
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        // Subscribe
        self.send_msg_to(
            connect.relay_url(),
            ClientMessage::new_req(id.clone(), vec![filter]),
        )
        .await?;

        let mut notifications = self.notifications();
        time::timeout(timeout, async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    if event.kind == Kind::NostrConnect {
                        let msg: String = nip04::decrypt(
                            &self.keys.secret_key()?,
                            &event.pubkey,
                            &event.content,
                        )?;
                        let msg = Message::from_json(msg)?;
                        if let Ok(Request::Connect(public_key)) = msg.to_request() {
                            connect.set_signer_public_key(public_key).await;
                            break;
                        }
                    }
                }
            }

            Ok::<(), Error>(())
        })
        .await
        .ok_or(Error::Timeout)??;

        // Unsubscribe
        self.send_msg_to(connect.relay_url(), ClientMessage::close(id))
            .await?;

        Ok(())
    }

    pub async fn send_request(
        &self,
        req: Request,
        timeout: Option<Duration>,
    ) -> Result<Response, Error> {
        let connect: &NostrConnect = self
            .connect
            .as_ref()
            .ok_or(Error::Client(super::Error::NIP46ClientNotConfigured))?;
        let signer_pubkey = connect
            .signer_public_key()
            .await
            .ok_or(Error::SignerPublicKeyNotFound)?;

        let msg = Message::request(req.clone());
        let req_id = msg.id();

        // Send request to signer
        let event =
            EventBuilder::nostr_connect(&self.keys, signer_pubkey, msg)?.to_event(&self.keys)?;
        self.send_event_to(connect.relay_url(), event).await?;

        let sub_id = SubscriptionId::generate();
        let filter = Filter::new()
            .pubkey(self.keys.public_key())
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        // Subscribe
        self.send_msg_to(
            connect.relay_url(),
            ClientMessage::new_req(sub_id.clone(), vec![filter]),
        )
        .await?;

        let mut notifications = self.notifications();
        let future = async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    if event.kind == Kind::NostrConnect {
                        let msg = nip04::decrypt(
                            &self.keys.secret_key()?,
                            &event.pubkey,
                            &event.content,
                        )?;
                        let msg = Message::from_json(msg)?;

                        log::debug!("New message received: {msg:#?}");

                        if let Message::Response { id, result, error } = &msg {
                            if &req_id == id {
                                if let Some(result) = result {
                                    let res = match req {
                                        Request::Describe => Response::Describe(
                                            serde_json::from_value(result.to_owned())?,
                                        ),
                                        Request::GetPublicKey => {
                                            let pubkey = serde_json::from_value(result.to_owned())?;
                                            Response::GetPublicKey(pubkey)
                                        }
                                        Request::SignEvent(_) => {
                                            let sig = serde_json::from_value(result.to_owned())?;
                                            Response::SignEvent(sig)
                                        }
                                        Request::Delegate { .. } => Response::Delegate(
                                            serde_json::from_value(result.to_owned())?,
                                        ),
                                        Request::Nip04Encrypt { .. } => Response::Nip04Encrypt(
                                            serde_json::from_value(result.to_owned())?,
                                        ),
                                        Request::Nip04Decrypt { .. } => Response::Nip04Decrypt(
                                            serde_json::from_value(result.to_owned())?,
                                        ),
                                        Request::SignSchnorr { .. } => Response::SignSchnorr(
                                            serde_json::from_value(result.to_owned())?,
                                        ),
                                        _ => break,
                                    };

                                    // Unsubscribe
                                    self.send_msg_to(
                                        connect.relay_url(),
                                        ClientMessage::close(sub_id.clone()),
                                    )
                                    .await?;
                                    return Ok(res);
                                }

                                if let Some(error) = error {
                                    // Unsubscribe
                                    self.send_msg_to(
                                        connect.relay_url(),
                                        ClientMessage::close(sub_id.clone()),
                                    )
                                    .await?;
                                    return Err(Error::Response(error.to_owned()));
                                }

                                break;
                            }
                        }
                    }
                }
            }

            Err(Error::Generic)
        };

        let res: Result<Response, Error> =
            time::timeout(timeout, future).await.ok_or(Error::Timeout)?;

        // Unsubscribe
        self.send_msg_to(connect.relay_url(), ClientMessage::close(sub_id))
            .await?;

        res
    }
}
