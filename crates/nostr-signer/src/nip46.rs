// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect (NIP46)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use std::sync::Arc;
use std::time::Duration;

use async_utility::time;
use nostr::nips::nip46::{self, Message, NostrConnectMetadata, NostrConnectURI, Request, Response};
use nostr::prelude::*;
use nostr::{key, serde_json};
use nostr_relay_pool::{
    FilterOptions, Relay, RelayNotification, RelayOptions, RelaySendOptions,
    SubscribeAutoCloseOptions, SubscribeOptions,
};
use thiserror::Error;
use tokio::sync::Mutex;

const TIMEOUT: Duration = Duration::from_secs(10);

/// Nostr Connect error
#[derive(Debug, Error)]
pub enum Error {
    /// Json
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Keys error
    #[error(transparent)]
    Keys(#[from] key::Error),
    /// Event builder error
    #[error(transparent)]
    Builder(#[from] builder::Error),
    /// NIP04 error
    #[error(transparent)]
    NIP04(#[from] nip04::Error),
    /// NIP46 error
    #[error(transparent)]
    NIP46(#[from] nip46::Error),
    /// Relay
    #[error(transparent)]
    Relay(#[from] nostr_relay_pool::relay::Error),
    /// Generic NIP46 error
    #[error("generic error")]
    Generic,
    /// NIP46 response error
    #[error("response error: {0}")]
    Response(String),
    /// Signer public key not found
    #[error("signer public key not found")]
    SignerPublicKeyNotFound,
    /// Request timeout
    #[error("timeout")]
    Timeout,
}

/// NIP46 Signer
#[derive(Debug, Clone)]
pub struct Nip46Signer {
    app_keys: Keys,
    signer_public_key: Arc<Mutex<Option<PublicKey>>>,
    relay: Relay,
    timeout: Duration,
}

impl Nip46Signer {
    /// New NIP46 remote signer
    pub async fn new(
        relay_url: Url,
        app_keys: Keys,
        signer_public_key: Option<PublicKey>,
        timeout: Duration,
    ) -> Result<Self, Error> {
        Self::with_opts(
            relay_url,
            app_keys,
            signer_public_key,
            timeout,
            RelayOptions::default(),
        )
        .await
    }

    /// New NIP46 remote signer
    pub async fn with_opts(
        relay_url: Url,
        app_keys: Keys,
        signer_public_key: Option<PublicKey>,
        timeout: Duration,
        opts: RelayOptions,
    ) -> Result<Self, Error> {
        // Compose pool
        let relay = Relay::with_opts(relay_url, opts);
        relay.connect(Some(Duration::from_secs(10))).await;

        Ok(Self {
            app_keys,
            signer_public_key: Arc::new(Mutex::new(signer_public_key)),
            relay,
            timeout,
        })
    }

    /// Get signer relay [`Url`]
    pub fn relay_url(&self) -> Url {
        self.relay.url()
    }

    /// Get signer [PublicKey]
    pub async fn signer_public_key(&self) -> Result<PublicKey, Error> {
        let mut signer_public_key = self.signer_public_key.lock().await;
        match *signer_public_key {
            Some(p) => Ok(p),
            None => {
                let public_key: PublicKey = self.get_signer_public_key().await?;
                *signer_public_key = Some(public_key);
                Ok(public_key)
            }
        }
    }

    /// Compose Nostr Connect URI
    pub fn nostr_connect_uri(&self, metadata: NostrConnectMetadata) -> NostrConnectURI {
        NostrConnectURI::with_metadata(self.app_keys.public_key(), self.relay_url(), metadata)
    }

    async fn get_signer_public_key(&self) -> Result<PublicKey, Error> {
        let public_key = self.app_keys.public_key();
        let secret_key = self.app_keys.secret_key()?;

        let filter = Filter::new()
            .pubkey(public_key)
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        let auto_close_opts = SubscribeAutoCloseOptions::default()
            .filter(FilterOptions::WaitForEventsAfterEOSE(1))
            .timeout(Some(TIMEOUT));
        let subscribe_opts = SubscribeOptions::default().close_on(Some(auto_close_opts));

        // Subscribe
        let id: SubscriptionId = self.relay.subscribe(vec![filter], subscribe_opts).await?;

        let mut notifications = self.relay.notifications();
        time::timeout(Some(self.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayNotification::Event {
                    subscription_id,
                    event,
                } = notification
                {
                    if subscription_id == id && event.kind() == Kind::NostrConnect {
                        let msg: String =
                            nip04::decrypt(secret_key, event.author_ref(), event.content())?;
                        let msg = Message::from_json(msg)?;
                        if let Ok(Request::Connect(pk)) = msg.to_request() {
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

        let signer_public_key: PublicKey = self.signer_public_key().await?;

        // Build request
        let event = EventBuilder::nostr_connect(&self.app_keys, signer_public_key, msg)?
            .to_event(&self.app_keys)?;

        let filter = Filter::new()
            .pubkey(public_key)
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        let auto_close_opts = SubscribeAutoCloseOptions::default()
            .filter(FilterOptions::WaitForEventsAfterEOSE(1))
            .timeout(Some(TIMEOUT));
        let subscribe_opts = SubscribeOptions::default().close_on(Some(auto_close_opts));

        // Subscribe
        let sub_id: SubscriptionId = self.relay.subscribe(vec![filter], subscribe_opts).await?;

        let mut notifications = self.relay.notifications();

        // Send request
        self.relay
            .send_event(event, RelaySendOptions::new())
            .await?;

        let future = async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayNotification::Event {
                    subscription_id,
                    event,
                } = notification
                {
                    if subscription_id == sub_id && event.kind() == Kind::NostrConnect {
                        let msg = nip04::decrypt(secret_key, event.author_ref(), event.content())?;
                        let msg = Message::from_json(msg)?;

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

                                    return Ok(res);
                                }

                                if let Some(error) = error {
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

        time::timeout(Some(timeout.unwrap_or(self.timeout)), future)
            .await
            .ok_or(Error::Timeout)?
    }

    /// Completely shutdown
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.relay.terminate().await?)
    }
}
