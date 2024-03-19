// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect (NIP46)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_utility::time;
use nostr::nips::nip46::{
    self, Message, NostrConnectMetadata, NostrConnectURI, Request, ResponseResult,
};
use nostr::prelude::*;
use nostr::{key, serde_json};
use nostr_relay_pool::{
    RelayOptions, RelayPool, RelayPoolNotification, RelaySendOptions, SubscribeOptions,
};
use thiserror::Error;
use tokio::sync::Mutex;

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
    /// Pool
    #[error(transparent)]
    Pool(#[from] nostr_relay_pool::pool::Error),
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
    uri: NostrConnectURI,
    app_keys: Keys,
    signer_public_key: Arc<Mutex<Option<PublicKey>>>,
    pool: RelayPool,
    timeout: Duration,
}

impl Nip46Signer {
    /// New NIP46 remote signer
    pub async fn new(
        uri: NostrConnectURI,
        app_keys: Option<Keys>,
        timeout: Duration,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error> {
        // Compose pool
        let pool: RelayPool = RelayPool::default();

        let opts: RelayOptions = opts.unwrap_or_default();
        for url in uri.relays().into_iter() {
            pool.add_relay(url, opts.clone()).await?;
        }

        pool.connect(Some(Duration::from_secs(10))).await;

        let app_keys: Keys = match app_keys {
            Some(keys) => keys,
            None => Keys::generate(),
        };
        let signer_public_key: Option<PublicKey> = uri.signer_public_key();

        let this = Self {
            uri,
            app_keys,
            signer_public_key: Arc::new(Mutex::new(signer_public_key)),
            pool,
            timeout,
        };

        this.subscribe().await;
        this.connect().await?;

        Ok(this)
    }

    /// Get local app keys
    pub fn local_keys(&self) -> &Keys {
        &self.app_keys
    }

    /// Get signer relays
    pub async fn relays(&self) -> Vec<Url> {
        self.pool.relays().await.into_keys().collect()
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
    pub async fn nostr_connect_uri(&self, metadata: NostrConnectMetadata) -> NostrConnectURI {
        NostrConnectURI::Client {
            public_key: self.app_keys.public_key(),
            relays: self.relays().await,
            metadata,
        }
    }

    async fn subscribe(&self) {
        let public_key: PublicKey = self.app_keys.public_key();

        let filter = Filter::new()
            .pubkey(public_key)
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        // Subscribe
        self.pool
            .subscribe(vec![filter], SubscribeOptions::default())
            .await;
    }

    async fn send_request(&self, req: Request) -> Result<ResponseResult, Error> {
        let secret_key: &SecretKey = self.app_keys.secret_key()?;
        let signer_public_key: PublicKey = self.signer_public_key().await?;

        // Convert request to event
        let msg = Message::request(req);
        tracing::debug!("Sending '{msg}' NIP46 message");

        let req_id = msg.id().to_string();
        let event: Event = EventBuilder::nostr_connect(&self.app_keys, signer_public_key, msg)?
            .to_event(&self.app_keys)?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool.send_event(event, RelaySendOptions::new()).await?;

        time::timeout(Some(self.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::NostrConnect {
                        let msg = nip04::decrypt(secret_key, event.author_ref(), event.content())?;
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
    async fn connect(&self) -> Result<(), Error> {
        let req = Request::Connect {
            public_key: self.app_keys.public_key(),
            secret: self.uri.secret(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_connect()?)
    }

    /// Sign an [UnsignedEvent]
    pub async fn get_relays(&self) -> Result<HashMap<Url, RelayPermissions>, Error> {
        let req = Request::GetRelays;
        let res = self.send_request(req).await?;
        Ok(res.to_get_relays()?)
    }

    /// Sign an [UnsignedEvent]
    pub async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let req = Request::SignEvent(unsigned);
        let res = self.send_request(req).await?;
        Ok(res.to_sign_event()?)
    }

    /// NIP04 encrypt
    pub async fn nip04_encrypt<T>(&self, public_key: PublicKey, content: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let content: &[u8] = content.as_ref();
        let req = Request::Nip04Encrypt {
            public_key,
            text: String::from_utf8_lossy(content).to_string(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// NIP04 decrypt
    pub async fn nip04_decrypt<S>(
        &self,
        public_key: PublicKey,
        ciphertext: S,
    ) -> Result<String, Error>
    where
        S: Into<String>,
    {
        let req = Request::Nip04Decrypt {
            public_key,
            ciphertext: ciphertext.into(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// NIP44 encrypt
    pub async fn nip44_encrypt<T>(&self, public_key: PublicKey, content: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let content: &[u8] = content.as_ref();
        let req = Request::Nip44Encrypt {
            public_key,
            text: String::from_utf8_lossy(content).to_string(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// NIP44 decrypt
    pub async fn nip44_decrypt<T>(&self, public_key: PublicKey, payload: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let payload: &[u8] = payload.as_ref();
        let req = Request::Nip44Decrypt {
            public_key,
            ciphertext: String::from_utf8_lossy(payload).to_string(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    async fn get_signer_public_key(&self) -> Result<PublicKey, Error> {
        let secret_key = self.app_keys.secret_key()?;

        let mut notifications = self.pool.notifications();
        time::timeout(Some(self.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::NostrConnect {
                        let msg: String =
                            nip04::decrypt(secret_key, event.author_ref(), event.content())?;
                        let msg = Message::from_json(msg)?;
                        if let Ok(Request::Connect { public_key, .. }) = msg.to_request() {
                            return Ok(public_key);
                        }
                    }
                }
            }

            Err(Error::SignerPublicKeyNotFound)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Completely shutdown
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.pool.shutdown().await?)
    }
}
