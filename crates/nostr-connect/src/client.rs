// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect client

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use async_utility::time;
use nostr::nips::nip46::{Message, NostrConnectURI, Request, ResponseResult};
use nostr::prelude::*;
use nostr_relay_pool::{
    RelayOptions, RelayPool, RelayPoolNotification, RelaySendOptions, SubscribeOptions,
};
use tokio::sync::broadcast::Receiver;
use tokio::sync::OnceCell;

use crate::error::Error;

#[allow(missing_docs)]
#[deprecated(since = "0.36.0", note = "Use `NostrConnect` instead")]
pub type Nip46Signer = NostrConnect;

/// Nostr Connect Client
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Debug, Clone)]
pub struct NostrConnect {
    app_keys: Keys,
    uri: NostrConnectURI,
    signer_public_key: OnceCell<PublicKey>,
    pool: RelayPool,
    timeout: Duration,
    opts: RelayOptions,
    secret: Option<String>,
    bootstrapped: Arc<AtomicBool>,
}

impl NostrConnect {
    /// Construct Nostr Connect client
    pub fn new(
        uri: NostrConnectURI,
        app_keys: Keys,
        timeout: Duration,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error> {
        // Check app keys
        if let NostrConnectURI::Client { public_key, .. } = &uri {
            if *public_key != app_keys.public_key() {
                return Err(Error::PublicKeyNotMatchAppKeys);
            }
        }

        // Get signer public key
        let signer_public_key: OnceCell<PublicKey> = match uri.signer_public_key() {
            Some(public_key) => OnceCell::from(public_key),
            None => OnceCell::new(),
        };

        Ok(Self {
            app_keys,
            signer_public_key,
            pool: RelayPool::default(),
            timeout,
            opts: opts.unwrap_or_default(),
            secret: uri.secret(),
            uri,
            bootstrapped: Arc::new(AtomicBool::new(false)),
        })
    }

    async fn bootstrap(&self) -> Result<PublicKey, Error> {
        // Add relays
        for url in self.uri.relays().into_iter() {
            self.pool.add_relay(url, self.opts.clone()).await?;
        }

        // Connect to relays
        self.pool.connect(None).await;

        // Subscribe
        let notifications = self.subscribe().await?;

        // Get signer public key
        let signer_public_key: PublicKey = match self.uri.signer_public_key() {
            Some(public_key) => public_key,
            None => get_signer_public_key(&self.app_keys, notifications, self.timeout).await?,
        };

        // Send `connect` command if bunker URI
        if self.uri.is_bunker() {
            self.connect(signer_public_key).await?;
        }

        self.bootstrapped.store(true, Ordering::SeqCst);

        Ok(signer_public_key)
    }

    async fn subscribe(&self) -> Result<Receiver<RelayPoolNotification>, Error> {
        let public_key: PublicKey = self.app_keys.public_key();

        let filter = Filter::new()
            .pubkey(public_key)
            .kind(Kind::NostrConnect)
            .limit(0);

        let notifications = self.pool.notifications();

        // Subscribe
        self.pool
            .subscribe(vec![filter], SubscribeOptions::default())
            .await?;

        Ok(notifications)
    }

    /// Get local app keys
    #[inline]
    pub fn local_keys(&self) -> &Keys {
        &self.app_keys
    }

    /// Get signer relays
    #[inline]
    pub fn relays(&self) -> Vec<Url> {
        self.uri.relays()
    }

    /// Get signer [PublicKey]
    #[inline]
    pub async fn signer_public_key(&self) -> Result<&PublicKey, Error> {
        // The bootstrap here is executed only if URI is NOT `bunker://`
        self.signer_public_key
            .get_or_try_init(|| async { self.bootstrap().await })
            .await
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> Result<NostrConnectURI, Error> {
        Ok(NostrConnectURI::Bunker {
            signer_public_key: *self.signer_public_key().await?,
            relays: self.relays(),
            secret: self.secret.clone(),
        })
    }

    #[inline]
    async fn send_request(&self, req: Request) -> Result<ResponseResult, Error> {
        // Get signer public key
        let signer_public_key: PublicKey = *self.signer_public_key().await?;

        // Check if bootstrap is executed
        // If it's not executed, bootstrap.
        if !self.bootstrapped.load(Ordering::SeqCst) {
            self.bootstrap().await?;
        }

        // Send request
        self.send_request_with_pk(req, signer_public_key).await
    }

    async fn send_request_with_pk(
        &self,
        req: Request,
        signer_public_key: PublicKey,
    ) -> Result<ResponseResult, Error> {
        let secret_key: &SecretKey = self.app_keys.secret_key();

        // Convert request to event
        let msg = Message::request(req);
        tracing::debug!("Sending '{msg}' NIP46 message");

        let req_id = msg.id().to_string();
        let event: Event = EventBuilder::nostr_connect(&self.app_keys, signer_public_key, msg)?
            .sign_with_keys(&self.app_keys)?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool.send_event(event, RelaySendOptions::new()).await?;

        time::timeout(Some(self.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::NostrConnect {
                        let msg = nip04::decrypt(secret_key, &event.pubkey, &event.content)?;
                        let msg = Message::from_json(msg)?;

                        tracing::debug!("Received NIP46 message: '{msg}'");

                        if let Message::Response { id, result, error } = &msg {
                            if &req_id == id {
                                if msg.is_auth_url() {
                                    tracing::warn!("Received 'auth_url': {error:?}");
                                } else {
                                    if let Some(result) = result {
                                        return Ok(result.clone());
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
            }

            Err(Error::Timeout)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Connect msg
    async fn connect(&self, signer_public_key: PublicKey) -> Result<(), Error> {
        let req = Request::Connect {
            public_key: signer_public_key,
            secret: self.secret.clone(),
        };
        let res = self.send_request_with_pk(req, signer_public_key).await?;
        Ok(res.to_connect()?)
    }

    /// Sign an [UnsignedEvent]
    pub async fn get_relays(&self) -> Result<HashMap<Url, RelayPermissions>, Error> {
        let req = Request::GetRelays;
        let res = self.send_request(req).await?;
        Ok(res.to_get_relays()?)
    }

    /// Sign an [UnsignedEvent]
    async fn _sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let req = Request::SignEvent(unsigned);
        let res = self.send_request(req).await?;
        Ok(res.to_sign_event()?)
    }

    async fn _nip04_encrypt(
        &self,
        public_key: PublicKey,
        content: String,
    ) -> Result<String, Error> {
        let req = Request::Nip04Encrypt {
            public_key,
            text: content,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    async fn _nip04_decrypt(
        &self,
        public_key: PublicKey,
        ciphertext: String,
    ) -> Result<String, Error> {
        let req = Request::Nip04Decrypt {
            public_key,
            ciphertext,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    async fn _nip44_encrypt(
        &self,
        public_key: PublicKey,
        content: String,
    ) -> Result<String, Error> {
        let req = Request::Nip44Encrypt {
            public_key,
            text: content,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    async fn _nip44_decrypt(
        &self,
        public_key: PublicKey,
        payload: String,
    ) -> Result<String, Error> {
        let req = Request::Nip44Decrypt {
            public_key,
            ciphertext: payload,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// Completely shutdown
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.pool.shutdown().await?)
    }
}

async fn get_signer_public_key(
    app_keys: &Keys,
    mut notifications: Receiver<RelayPoolNotification>,
    timeout: Duration,
) -> Result<PublicKey, Error> {
    time::timeout(Some(timeout), async {
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::NostrConnect {
                    // Decrypt content
                    let msg: String =
                        nip04::decrypt(app_keys.secret_key(), &event.pubkey, event.content)?;

                    tracing::debug!("Received Nostr Connect message: '{msg}'");

                    // Parse message
                    let msg: Message = Message::from_json(msg)?;

                    // Match message
                    match msg {
                        Message::Request {
                            req: Request::Connect { public_key, .. },
                            ..
                        } => {
                            return Ok(public_key);
                        }
                        Message::Response {
                            result: Some(ResponseResult::Connect),
                            error: None,
                            ..
                        } => return Ok(event.pubkey),
                        _ => {}
                    }
                }
            }
        }

        Err(Error::SignerPublicKeyNotFound)
    })
    .await
    .ok_or(Error::Timeout)?
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrSigner for NostrConnect {
    async fn get_public_key(&self) -> Result<PublicKey, SignerError> {
        // TODO: avoid copied?
        self.signer_public_key()
            .await
            .map_err(SignerError::backend)
            .copied()
    }

    async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError> {
        self._sign_event(unsigned)
            .await
            .map_err(SignerError::backend)
    }

    async fn nip04_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip04_encrypt(*public_key, content.to_string())
            .await
            .map_err(SignerError::backend)
    }

    async fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip04_decrypt(*public_key, content.to_string())
            .await
            .map_err(SignerError::backend)
    }

    async fn nip44_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip44_encrypt(*public_key, content.to_string())
            .await
            .map_err(SignerError::backend)
    }

    async fn nip44_decrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip44_decrypt(*public_key, content.to_string())
            .await
            .map_err(SignerError::backend)
    }
}
