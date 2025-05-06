// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect client

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_utility::time;
use nostr::nips::nip46::ResponseResult;
use nostr_relay_pool::prelude::*;
use tokio::sync::broadcast::Receiver;
use tokio::sync::OnceCell;

use crate::error::Error;

/// Nostr Connect Client
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Debug, Clone)]
pub struct NostrConnect {
    uri: NostrConnectURI,
    app_keys: Keys,
    remote_signer_public_key: OnceCell<PublicKey>,
    user_public_key: OnceCell<PublicKey>,
    pool: RelayPool,
    timeout: Duration,
    opts: RelayOptions,
    secret: Option<String>,
    auth_url_handler: Option<Arc<dyn AuthUrlHandler>>,
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
            if public_key != &app_keys.public_key {
                return Err(Error::PublicKeyNotMatchAppKeys);
            }
        }

        Ok(Self {
            app_keys,
            // NOT set the remote_signer_public_key, also if bunker URI!
            // If you already set remote_signer_public_key, you'll need another field to know if boostrap was already done.
            remote_signer_public_key: OnceCell::new(),
            user_public_key: OnceCell::new(),
            pool: RelayPool::default(),
            timeout,
            opts: opts.unwrap_or_default(),
            secret: uri.secret().map(|secret| secret.to_string()),
            uri,
            auth_url_handler: None,
        })
    }

    /// Set an `auth_url` handler
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// use nostr_connect::prelude::*;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyAuthUrlHandler;
    ///
    /// impl AuthUrlHandler for MyAuthUrlHandler {
    ///     fn on_auth_url(&self, auth_url: Url) -> BoxedFuture<Result<()>> {
    ///         Box::pin(async move {
    ///         webbrowser::open(auth_url.as_str())?;
    ///             Ok(())
    ///         })
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let uri = NostrConnectURI::parse("bunker://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=wss://relay.nsec.app")?;
    ///     let app_keys = Keys::generate();
    ///     let timeout = Duration::from_secs(60);
    ///
    ///     let mut connect = NostrConnect::new(uri, app_keys, timeout, None)?;
    ///
    ///     // Set auth_url handler
    ///     connect.auth_url_handler(MyAuthUrlHandler);
    ///
    ///     // ...
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn auth_url_handler<T>(&mut self, handler: T)
    where
        T: IntoAuthUrlHandler,
    {
        self.auth_url_handler = Some(handler.into_auth_url_handler());
    }

    /// Get relays status
    pub async fn status(&self) -> HashMap<RelayUrl, RelayStatus> {
        let relays = self.pool.relays().await;
        relays.into_iter().map(|(u, r)| (u, r.status())).collect()
    }

    async fn bootstrap(&self) -> Result<PublicKey, Error> {
        // Add relays
        for url in self.uri.relays().iter() {
            self.pool.add_relay(url, self.opts.clone()).await?;
        }

        // Connect to relays
        self.pool.connect().await;

        // Subscribe
        let notifications = self.subscribe().await?;

        // Get remote signer public key
        let remote_signer_public_key: PublicKey = match self.uri.remote_signer_public_key() {
            Some(public_key) => *public_key,
            None => {
                match get_remote_signer_public_key(&self.app_keys, notifications, self.timeout)
                    .await?
                {
                    GetRemoteSignerPublicKey::RemoteOnly(public_key) => public_key,
                    GetRemoteSignerPublicKey::WithUserPublicKey { remote, user } => {
                        // Check if user public key was already set
                        match self.user_public_key.get().copied() {
                            Some(set_user_public_key) => {
                                // User public key was already set but not match the one received by the signer.
                                if set_user_public_key != user {
                                    return Err(Error::UserPublicKeyNotMatch {
                                        expected: Box::new(user),
                                        local: Box::new(set_user_public_key),
                                    });
                                }
                            }
                            None => {
                                // No user public key in cell, set it.
                                self.user_public_key.set(user)?;
                            }
                        }

                        // Return remote signer public key
                        remote
                    }
                }
            }
        };

        // Send `connect` command if bunker URI
        if self.uri.is_bunker() {
            self.connect(remote_signer_public_key).await?;
        }

        Ok(remote_signer_public_key)
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
            .subscribe(filter, SubscribeOptions::default())
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
    pub fn relays(&self) -> &[RelayUrl] {
        self.uri.relays()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> Result<NostrConnectURI, Error> {
        Ok(NostrConnectURI::Bunker {
            remote_signer_public_key: *self.remote_signer_public_key().await?,
            relays: self.relays().to_vec(),
            secret: self.secret.clone(),
        })
    }

    /// Manually set the user public key
    ///
    /// Be cautious when using this method, as providing an incorrect [`PublicKey`] can lead to potential issues.
    ///
    /// According to [`NIP46`](https://github.com/nostr-protocol/nips/blob/926a51e72206dfa71b9950a0ce58b54a7422aad2/46.md) the user public key must be requested from the signer.
    /// Once you set the user public key with this method, it can't be longer changed ans you'll have to create a new client.
    /// This method is intended to be used only if you are certain that public key stored by the signer match your (stored) one.
    #[inline]
    pub fn non_secure_set_user_public_key(&self, user_public_key: PublicKey) -> Result<(), Error> {
        Ok(self.user_public_key.set(user_public_key)?)
    }

    #[inline]
    async fn remote_signer_public_key(&self) -> Result<&PublicKey, Error> {
        self.remote_signer_public_key
            .get_or_try_init(|| async { self.bootstrap().await })
            .await
    }

    #[inline]
    async fn send_request(&self, req: NostrConnectRequest) -> Result<ResponseResult, Error> {
        // Get remote signer public key
        let remote_signer_public_key: PublicKey = *self.remote_signer_public_key().await?;

        // Send request
        self.send_request_with_pk(req, remote_signer_public_key)
            .await
    }

    async fn send_request_with_pk(
        &self,
        req: NostrConnectRequest,
        remote_signer_public_key: PublicKey,
    ) -> Result<ResponseResult, Error> {
        let secret_key: &SecretKey = self.app_keys.secret_key();

        // Convert request to event
        let msg = NostrConnectMessage::request(&req);
        tracing::debug!("Sending '{msg}' NIP46 message");

        let req_id = msg.id().to_string();
        let event: Event =
            EventBuilder::nostr_connect(&self.app_keys, remote_signer_public_key, msg)?
                .sign_with_keys(&self.app_keys)?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool.send_event(&event).await?;

        time::timeout(Some(self.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::NostrConnect {
                        let msg: String =
                            nip44::decrypt(secret_key, &event.pubkey, event.content.as_str())?;
                        let msg: NostrConnectMessage = NostrConnectMessage::from_json(msg)?;

                        tracing::debug!("Received NIP46 message: '{msg}'");

                        if req_id == msg.id() && msg.is_response() {
                            let response: NostrConnectResponse = msg.to_response(req.method())?;

                            if response.is_auth_url() {
                                if let (Some(auth_url), Some(handler)) =
                                    (response.error, &self.auth_url_handler)
                                {
                                    match Url::parse(&auth_url) {
                                        Ok(url) => {
                                            if let Err(e) = handler.on_auth_url(url).await {
                                                tracing::error!(
                                                    "Impossible to handle `auth_url`: {e}"
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Can't parse `auth_url`: {e}")
                                        }
                                    }
                                }
                            } else {
                                if let Some(error) = response.error {
                                    return Err(Error::Response(error));
                                }

                                if let Some(result) = response.result {
                                    return Ok(result);
                                }

                                break;
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
    async fn connect(&self, remote_signer_public_key: PublicKey) -> Result<(), Error> {
        let req = NostrConnectRequest::Connect {
            public_key: remote_signer_public_key,
            secret: self.secret.clone(),
        };
        let res = self
            .send_request_with_pk(req, remote_signer_public_key)
            .await?;
        Ok(res.to_ack()?)
    }

    /// Sign an [UnsignedEvent]
    pub async fn get_relays(&self) -> Result<HashMap<RelayUrl, RelayPermissions>, Error> {
        let req = NostrConnectRequest::GetRelays;
        let res = self.send_request(req).await?;
        Ok(res.to_get_relays()?)
    }

    async fn _get_public_key(&self) -> Result<&PublicKey, Error> {
        self.user_public_key
            .get_or_try_init(|| async {
                let res = self.send_request(NostrConnectRequest::GetPublicKey).await?;
                Ok(res.to_get_public_key()?)
            })
            .await
    }

    /// Sign an [UnsignedEvent]
    async fn _sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let req = NostrConnectRequest::SignEvent(unsigned);
        let res = self.send_request(req).await?;
        Ok(res.to_sign_event()?)
    }

    async fn _nip04_encrypt(
        &self,
        public_key: PublicKey,
        content: String,
    ) -> Result<String, Error> {
        let req = NostrConnectRequest::Nip04Encrypt {
            public_key,
            text: content,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_nip04_encrypt()?)
    }

    async fn _nip04_decrypt(
        &self,
        public_key: PublicKey,
        ciphertext: String,
    ) -> Result<String, Error> {
        let req = NostrConnectRequest::Nip04Decrypt {
            public_key,
            ciphertext,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_nip04_decrypt()?)
    }

    async fn _nip44_encrypt(
        &self,
        public_key: PublicKey,
        content: String,
    ) -> Result<String, Error> {
        let req = NostrConnectRequest::Nip44Encrypt {
            public_key,
            text: content,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_nip44_encrypt()?)
    }

    async fn _nip44_decrypt(
        &self,
        public_key: PublicKey,
        payload: String,
    ) -> Result<String, Error> {
        let req = NostrConnectRequest::Nip44Decrypt {
            public_key,
            ciphertext: payload,
        };
        let res = self.send_request(req).await?;
        Ok(res.to_nip44_decrypt()?)
    }

    /// Completely shutdown
    pub async fn shutdown(self) {
        self.pool.shutdown().await
    }
}

enum GetRemoteSignerPublicKey {
    WithUserPublicKey { remote: PublicKey, user: PublicKey },
    RemoteOnly(PublicKey),
}

async fn get_remote_signer_public_key(
    app_keys: &Keys,
    mut notifications: Receiver<RelayPoolNotification>,
    timeout: Duration,
) -> Result<GetRemoteSignerPublicKey, Error> {
    time::timeout(Some(timeout), async {
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::NostrConnect {
                    // Decrypt content
                    let msg: String = nip44::decrypt(
                        app_keys.secret_key(),
                        &event.pubkey,
                        event.content.as_str(),
                    )?;

                    tracing::debug!("Received Nostr Connect message: '{msg}'");

                    // Parse message
                    let msg: NostrConnectMessage = NostrConnectMessage::from_json(msg)?;

                    // Match message
                    match &msg {
                        NostrConnectMessage::Request { .. } => {
                            if let Ok(NostrConnectRequest::Connect { public_key, .. }) =
                                msg.to_request()
                            {
                                return Ok(GetRemoteSignerPublicKey::WithUserPublicKey {
                                    remote: event.pubkey,
                                    user: public_key,
                                });
                            }
                        }
                        NostrConnectMessage::Response { .. } => {
                            if let Ok(NostrConnectResponse {
                                result: Some(ResponseResult::Ack),
                                error: None,
                            }) = msg.to_response(NostrConnectMethod::Connect)
                            {
                                return Ok(GetRemoteSignerPublicKey::RemoteOnly(event.pubkey));
                            }
                        }
                    }
                }
            }
        }

        Err(Error::SignerPublicKeyNotFound)
    })
    .await
    .ok_or(Error::Timeout)?
}

/// Nostr Connect auth_url handler
pub trait AuthUrlHandler: fmt::Debug + Send + Sync {
    /// Handle `auth_url` message
    fn on_auth_url(&self, auth_url: Url) -> BoxedFuture<Result<()>>;
}

#[doc(hidden)]
pub trait IntoAuthUrlHandler {
    fn into_auth_url_handler(self) -> Arc<dyn AuthUrlHandler>;
}

impl<T> IntoAuthUrlHandler for T
where
    T: AuthUrlHandler + 'static,
{
    fn into_auth_url_handler(self) -> Arc<dyn AuthUrlHandler> {
        Arc::new(self)
    }
}

impl NostrSigner for NostrConnect {
    fn backend(&self) -> SignerBackend {
        SignerBackend::NostrConnect
    }

    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>> {
        Box::pin(async move {
            self._get_public_key()
                .await
                .copied()
                .map_err(SignerError::backend)
        })
    }

    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>> {
        Box::pin(async move {
            self._sign_event(unsigned)
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip04_encrypt(*public_key, content.to_string())
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip04_decrypt(*public_key, content.to_string())
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip44_encrypt(*public_key, content.to_string())
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip44_decrypt(*public_key, content.to_string())
                .await
                .map_err(SignerError::backend)
        })
    }
}
