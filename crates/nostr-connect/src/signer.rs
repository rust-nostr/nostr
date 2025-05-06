// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect signer

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nostr::nips::nip46::ResponseResult;
use nostr_relay_pool::prelude::*;

use crate::error::Error;

/// Nostr Connect Keys
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NostrConnectKeys {
    /// The keys used for communication with the client.
    ///
    /// This may be the same as the `user` one.
    pub signer: Keys,
    /// The keys used to sign events and so on.
    pub user: Keys,
}

/// Nostr Connect Signer
///
/// Signer that listen for requests from a client, handle them and send the response.
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Debug, Clone)]
pub struct NostrConnectRemoteSigner {
    keys: NostrConnectKeys,
    relays: Vec<RelayUrl>,
    pool: RelayPool,
    opts: RelayOptions,
    secret: Option<String>,
    nostr_connect_client_public_key: Option<PublicKey>,
    bootstrapped: Arc<AtomicBool>,
}

impl NostrConnectRemoteSigner {
    /// Construct new remote signer
    pub fn new<I, U>(
        keys: NostrConnectKeys,
        urls: I,
        secret: Option<String>,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let mut relays = Vec::new();
        for relay in urls.into_iter() {
            relays.push(
                relay
                    .try_into_url()
                    .map_err(|e| Error::Pool(pool::Error::from(e)))?,
            );
        }

        Ok(Self {
            keys,
            relays,
            pool: RelayPool::default(),
            opts: opts.unwrap_or_default(),
            secret,
            nostr_connect_client_public_key: None,
            bootstrapped: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Construct remote signer from client URI (`nostrconnect://..`)
    pub fn from_uri(
        uri: NostrConnectURI,
        keys: NostrConnectKeys,
        secret: Option<String>,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error> {
        match uri {
            NostrConnectURI::Client {
                public_key, relays, ..
            } => {
                let mut signer = Self::new(keys, relays, secret, opts)?;
                signer.nostr_connect_client_public_key = Some(public_key);
                Ok(signer)
            }
            NostrConnectURI::Bunker { .. } => Err(Error::UnexpectedUri),
        }
    }

    /// Get signer relays
    pub fn relays(&self) -> &[RelayUrl] {
        &self.relays
    }

    /// Get `bunker` URI
    pub fn bunker_uri(&self) -> NostrConnectURI {
        NostrConnectURI::Bunker {
            remote_signer_public_key: self.keys.signer.public_key(),
            relays: self.relays().to_vec(),
            secret: self.secret.clone(),
        }
    }

    async fn send_connect_ack(&self, public_key: PublicKey) -> Result<(), Error> {
        let req: NostrConnectRequest = NostrConnectRequest::Connect {
            public_key: self.keys.user.public_key(),
            secret: self.secret.clone(),
        };

        let msg: NostrConnectMessage = NostrConnectMessage::request(&req);
        let event: Event = EventBuilder::nostr_connect(&self.keys.signer, public_key, msg)?
            .sign_with_keys(&self.keys.signer)?;
        self.pool.send_event(&event).await?;
        Ok(())
    }

    async fn bootstrap(&self) -> Result<(), Error> {
        // Check if already bootstrapped
        if self.bootstrapped.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Add relays to pool
        for url in self.relays.iter() {
            self.pool.add_relay(url, self.opts.clone()).await?;
        }

        // Connect
        self.pool.connect().await;

        let filter = Filter::new()
            .pubkey(self.keys.signer.public_key())
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        // Subscribe
        self.pool
            .subscribe(filter, SubscribeOptions::default())
            .await?;

        // Mark as bootstrapped
        self.bootstrapped.store(true, Ordering::SeqCst);

        Ok(())
    }

    fn match_secret(&self, secret: Option<String>) -> bool {
        match (&self.secret, secret) {
            // Both secrets are set, check if values are equal.
            (Some(s1), Some(s2)) => s1 == &s2,
            // Only the secret on our side is set, must return `false`.
            (Some(..), None) => false,
            // Only the secret on their side is set, can continue, return `true`.
            (None, Some(..)) => true,
            // No secret is set
            (None, None) => true,
        }
    }

    /// Serve signer
    pub async fn serve<T>(&self, actions: T) -> Result<(), Error>
    where
        T: NostrConnectSignerActions,
    {
        self.bootstrap().await?;

        // TODO: move into bootstrap method?
        if let Some(public_key) = self.nostr_connect_client_public_key {
            self.send_connect_ack(public_key).await?;
        }

        self.pool
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::NostrConnect {
                        match nip44::decrypt(
                            self.keys.signer.secret_key(),
                            &event.pubkey,
                            &event.content,
                        ) {
                            Ok(msg) => {
                                tracing::debug!("New Nostr Connect message received: {msg}");

                                let msg: NostrConnectMessage = NostrConnectMessage::from_json(msg)?;
                                let id: String = msg.id().to_string();

                                if let Ok(req) = msg.to_request() {
                                    // Generate response
                                    let response: NostrConnectResponse = if actions
                                        .approve(&event.pubkey, &req)
                                    {
                                        match req {
                                            NostrConnectRequest::Connect { secret, .. } => {
                                                if self.match_secret(secret) {
                                                    NostrConnectResponse::with_result(
                                                        ResponseResult::Ack,
                                                    )
                                                } else {
                                                    NostrConnectResponse::with_error(
                                                        "Secret not match",
                                                    )
                                                }
                                            }
                                            NostrConnectRequest::GetPublicKey => {
                                                NostrConnectResponse::with_result(
                                                    ResponseResult::GetPublicKey(
                                                        self.keys.user.public_key(),
                                                    ),
                                                )
                                            }
                                            NostrConnectRequest::GetRelays => {
                                                NostrConnectResponse::with_error(
                                                    "Not supported yet",
                                                )
                                            }
                                            NostrConnectRequest::Nip04Encrypt {
                                                public_key,
                                                text,
                                            } => {
                                                match nip04::encrypt(
                                                    self.keys.user.secret_key(),
                                                    &public_key,
                                                    text,
                                                ) {
                                                    Ok(ciphertext) => {
                                                        NostrConnectResponse::with_result(
                                                            ResponseResult::Nip04Encrypt {
                                                                ciphertext,
                                                            },
                                                        )
                                                    }
                                                    Err(e) => NostrConnectResponse::with_error(
                                                        e.to_string(),
                                                    ),
                                                }
                                            }
                                            NostrConnectRequest::Nip04Decrypt {
                                                public_key,
                                                ciphertext,
                                            } => {
                                                match nip04::decrypt(
                                                    self.keys.user.secret_key(),
                                                    &public_key,
                                                    ciphertext,
                                                ) {
                                                    Ok(plaintext) => {
                                                        NostrConnectResponse::with_result(
                                                            ResponseResult::Nip04Decrypt {
                                                                plaintext,
                                                            },
                                                        )
                                                    }
                                                    Err(e) => NostrConnectResponse::with_error(
                                                        e.to_string(),
                                                    ),
                                                }
                                            }
                                            NostrConnectRequest::Nip44Encrypt {
                                                public_key,
                                                text,
                                            } => {
                                                match nip44::encrypt(
                                                    self.keys.user.secret_key(),
                                                    &public_key,
                                                    text,
                                                    nip44::Version::default(),
                                                ) {
                                                    Ok(ciphertext) => {
                                                        NostrConnectResponse::with_result(
                                                            ResponseResult::Nip44Encrypt {
                                                                ciphertext,
                                                            },
                                                        )
                                                    }
                                                    Err(e) => NostrConnectResponse::with_error(
                                                        e.to_string(),
                                                    ),
                                                }
                                            }
                                            NostrConnectRequest::Nip44Decrypt {
                                                public_key,
                                                ciphertext,
                                            } => {
                                                match nip44::decrypt(
                                                    self.keys.user.secret_key(),
                                                    &public_key,
                                                    ciphertext,
                                                ) {
                                                    Ok(plaintext) => {
                                                        NostrConnectResponse::with_result(
                                                            ResponseResult::Nip44Decrypt {
                                                                plaintext,
                                                            },
                                                        )
                                                    }
                                                    Err(e) => NostrConnectResponse::with_error(
                                                        e.to_string(),
                                                    ),
                                                }
                                            }
                                            NostrConnectRequest::SignEvent(unsigned) => {
                                                match unsigned.sign_with_keys(&self.keys.user) {
                                                    Ok(event) => NostrConnectResponse::with_result(
                                                        ResponseResult::SignEvent(Box::new(event)),
                                                    ),
                                                    Err(e) => NostrConnectResponse::with_error(
                                                        e.to_string(),
                                                    ),
                                                }
                                            }
                                            NostrConnectRequest::Ping => {
                                                NostrConnectResponse::with_result(
                                                    ResponseResult::Pong,
                                                )
                                            }
                                        }
                                    } else {
                                        NostrConnectResponse::with_error("Rejected")
                                    };

                                    // Compose message
                                    let msg: NostrConnectMessage =
                                        NostrConnectMessage::response(id, response);

                                    // Compose and publish event
                                    let event = EventBuilder::nostr_connect(
                                        &self.keys.signer,
                                        event.pubkey,
                                        msg,
                                    )?
                                    .sign_with_keys(&self.keys.signer)?;
                                    self.pool.send_event(&event).await?;
                                }
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Impossible to decrypt message.")
                            }
                        }
                    }
                }
                Ok(false) // Set to true to exit from the loop
            })
            .await?;

        Ok(())
    }
}

/// Nostr Connect signer actions
pub trait NostrConnectSignerActions {
    /// Approve
    fn approve(&self, public_key: &PublicKey, req: &NostrConnectRequest) -> bool;
}
