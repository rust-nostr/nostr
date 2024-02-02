// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect Client signer (NIP46)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use std::time::Duration;

use async_utility::time;
use nostr::nips::nip46::{Message, NostrConnectMetadata, NostrConnectURI, Request, Response};
use nostr::prelude::*;
use nostr::serde_json;

use crate::client::{Client, Error};
use crate::relay::RelayPoolNotification;

/// NIP46 Signer
#[derive(Debug, Clone)]
pub struct Nip46Signer {
    relay_url: Url,
    app_keys: Keys,
    signer_public_key: XOnlyPublicKey,
    client: Client,
    timeout: Duration,
}

impl Nip46Signer {
    /// New NIP46 remote signer
    pub async fn new(
        relay_url: Url,
        app_keys: Keys,
        signer_public_key: Option<XOnlyPublicKey>,
        timeout: Duration,
    ) -> Result<Self, Error> {
        // Compose client
        let client = Client::new(&app_keys);
        client.add_relay(&relay_url).await?;
        client.connect().await;

        let signer_public_key: XOnlyPublicKey = match signer_public_key {
            Some(signer_public_key) => signer_public_key,
            None => get_signer_public_key(&relay_url, &app_keys, &client, timeout).await?,
        };

        Ok(Self {
            relay_url,
            app_keys,
            signer_public_key,
            client,
            timeout,
        })
    }

    /// Get signer relay [`Url`]
    pub fn relay_url(&self) -> Url {
        self.relay_url.clone()
    }

    /// Get signer [`XOnlyPublicKey`]
    pub fn signer_public_key(&self) -> XOnlyPublicKey {
        self.signer_public_key
    }

    /// Compose Nostr Connect URI
    pub fn nostr_connect_uri(&self, metadata: NostrConnectMetadata) -> NostrConnectURI {
        NostrConnectURI::with_metadata(self.app_keys.public_key(), self.relay_url(), metadata)
    }

    /// Send NIP46 [`Request`] to signer
    pub async fn send_req_to_signer(
        &self,
        req: Request,
        timeout: Option<Duration>,
    ) -> Result<Response, Error> {
        let msg = Message::request(req.clone());
        let req_id = msg.id();

        let public_key = self.app_keys.public_key();
        let secret_key = self.app_keys.secret_key()?;

        // Build request
        let event = EventBuilder::nostr_connect(&self.app_keys, self.signer_public_key, msg)?
            .to_event(&self.app_keys)?;

        // Send request to signer
        self.client.send_event_to([self.relay_url()], event).await?;

        let sub_id = SubscriptionId::generate();
        let filter = Filter::new()
            .pubkey(public_key)
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        // Subscribe
        self.client
            .send_msg_to(
                [self.relay_url()],
                ClientMessage::req(sub_id.clone(), vec![filter]),
            )
            .await?;

        let mut notifications = self.client.notifications();
        let future = async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::NostrConnect {
                        let msg = nip04::decrypt(&secret_key, event.author_ref(), event.content())?;
                        let msg = Message::from_json(msg)?;

                        tracing::debug!("New message received: {msg:?}");

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
                                    self.client
                                        .send_msg_to(
                                            [self.relay_url()],
                                            ClientMessage::close(sub_id.clone()),
                                        )
                                        .await?;
                                    return Ok(res);
                                }

                                if let Some(error) = error {
                                    // Unsubscribe
                                    self.client
                                        .send_msg_to(
                                            [self.relay_url()],
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
            time::timeout(Some(timeout.unwrap_or(self.timeout)), future)
                .await
                .ok_or(Error::Timeout)?;

        // Unsubscribe
        self.client
            .send_msg_to([self.relay_url()], ClientMessage::close(sub_id))
            .await?;

        res
    }
}

async fn get_signer_public_key(
    relay_url: &Url,
    app_keys: &Keys,
    client: &Client,
    timeout: Duration,
) -> Result<XOnlyPublicKey, Error> {
    let public_key = app_keys.public_key();
    let secret_key = app_keys.secret_key()?;

    let id = SubscriptionId::generate();
    let filter = Filter::new()
        .pubkey(public_key)
        .kind(Kind::NostrConnect)
        .since(Timestamp::now());

    // Subscribe
    client
        .send_msg_to([relay_url], ClientMessage::req(id.clone(), vec![filter]))
        .await?;

    let mut notifications = client.notifications();
    time::timeout(Some(timeout), async {
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind() == Kind::NostrConnect {
                    let msg: String =
                        nip04::decrypt(&secret_key, event.author_ref(), event.content())?;
                    let msg = Message::from_json(msg)?;
                    if let Ok(Request::Connect(pk)) = msg.to_request() {
                        // Unsubscribe
                        client
                            .send_msg_to([relay_url], ClientMessage::close(id))
                            .await?;
                        return Ok(pk);
                    }
                }
            }
        }

        Err(Error::SignerPublicKeyNotFound)
    })
    .await
    .ok_or(Error::Timeout)?
}
