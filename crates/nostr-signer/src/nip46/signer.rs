// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect signer

use std::time::Duration;

use nostr::nips::nip46::{Message, NostrConnectURI, Request, ResponseResult};
use nostr::prelude::*;
use nostr_relay_pool::{
    pool, RelayOptions, RelayPool, RelayPoolNotification, RelaySendOptions, SubscribeOptions,
};

use super::Error;

/// Nostr Connect Signer
///
/// Signer that listen for requests from client, handle them and send the response.
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Debug, Clone)]
pub struct NostrConnectRemoteSigner {
    keys: Keys,
    pool: RelayPool,
    secret: Option<String>,
}

impl NostrConnectRemoteSigner {
    /// Construct new remote signer
    pub async fn new<I, U>(
        secret_key: SecretKey,
        relays: I,
        secret: Option<String>,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        // Compose pool
        let pool: RelayPool = RelayPool::default();

        let opts: RelayOptions = opts.unwrap_or_default();
        for url in relays.into_iter() {
            pool.add_relay(url, opts.clone()).await?;
        }

        pool.connect(Some(Duration::from_secs(10))).await;

        Ok(Self {
            keys: Keys::new(secret_key),
            pool,
            secret,
        })
    }

    /// Construct remote signer from client URI (`nostrconnect://..`)
    pub async fn from_uri(
        uri: NostrConnectURI,
        secret_key: SecretKey,
        secret: Option<String>,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error> {
        match uri {
            NostrConnectURI::Client {
                public_key, relays, ..
            } => {
                let this = Self::new(secret_key, relays, secret, opts).await?;
                this.send_connect_ack(public_key).await?;
                Ok(this)
            }
            NostrConnectURI::Bunker { .. } => Err(Error::UnexpectedUri),
        }
    }

    /// Get signer relays
    pub async fn relays(&self) -> Vec<Url> {
        self.pool.relays().await.into_keys().collect()
    }

    /// Get Nostr Connect URI
    #[deprecated(since = "0.33.0", note = "Use `bunker_uri` instead")]
    pub async fn nostr_connect_uri(&self) -> NostrConnectURI {
        self.bunker_uri().await
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> NostrConnectURI {
        NostrConnectURI::Bunker {
            signer_public_key: self.keys.public_key(),
            relays: self.relays().await,
            secret: self.secret.clone(),
        }
    }

    async fn send_connect_ack(&self, public_key: PublicKey) -> Result<(), Error> {
        let msg = Message::request(Request::Connect {
            public_key: self.keys.public_key(),
            secret: None,
        });
        let event =
            EventBuilder::nostr_connect(&self.keys, public_key, msg)?.to_event(&self.keys)?;
        self.pool
            .send_event(event, RelaySendOptions::default())
            .await?;
        Ok(())
    }

    async fn subscribe(&self) -> Result<(), Error> {
        let public_key: PublicKey = self.keys.public_key();

        let filter = Filter::new()
            .pubkey(public_key)
            .kind(Kind::NostrConnect)
            .since(Timestamp::now());

        // Subscribe
        self.pool
            .subscribe(vec![filter], SubscribeOptions::default())
            .await?;

        Ok(())
    }

    /// Serve signer
    pub async fn serve<T>(&self, actions: T) -> Result<(), Error>
    where
        T: NostrConnectSignerActions,
    {
        self.subscribe().await?;

        self.pool
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::NostrConnect {
                        if let Ok(msg) = nip04::decrypt(
                            self.keys.secret_key()?,
                            event.author_ref(),
                            event.content(),
                        ) {
                            tracing::debug!("New Nostr Connect message received: {msg}");

                            let msg: Message = Message::from_json(msg)?;

                            if let Message::Request { id, req } = msg {
                                // Generate response
                                let (result, error) = if actions.approve(&req) {
                                    match req {
                                        Request::Connect { secret, .. } => {
                                            if secret.unwrap_or_default()
                                                == self.secret.clone().unwrap_or_default()
                                            {
                                                (Some(ResponseResult::Connect), None)
                                            } else {
                                                (None, Some(String::from("Secret not match")))
                                            }
                                        }
                                        Request::GetPublicKey => (
                                            Some(ResponseResult::GetPublicKey(
                                                self.keys.public_key(),
                                            )),
                                            None,
                                        ),
                                        Request::GetRelays => {
                                            (None, Some(String::from("Not supported yet")))
                                        }
                                        Request::Nip04Encrypt { public_key, text } => {
                                            match nip04::encrypt(
                                                self.keys.secret_key()?,
                                                &public_key,
                                                text,
                                            ) {
                                                Ok(ciphertext) => (
                                                    Some(ResponseResult::EncryptionDecryption(
                                                        ciphertext,
                                                    )),
                                                    None,
                                                ),
                                                Err(e) => (None, Some(e.to_string())),
                                            }
                                        }
                                        Request::Nip04Decrypt {
                                            public_key,
                                            ciphertext,
                                        } => {
                                            match nip04::decrypt(
                                                self.keys.secret_key()?,
                                                &public_key,
                                                ciphertext,
                                            ) {
                                                Ok(ciphertext) => (
                                                    Some(ResponseResult::EncryptionDecryption(
                                                        ciphertext,
                                                    )),
                                                    None,
                                                ),
                                                Err(e) => (None, Some(e.to_string())),
                                            }
                                        }
                                        Request::Nip44Encrypt { public_key, text } => {
                                            match nip44::encrypt(
                                                self.keys.secret_key()?,
                                                &public_key,
                                                text,
                                                nip44::Version::default(),
                                            ) {
                                                Ok(ciphertext) => (
                                                    Some(ResponseResult::EncryptionDecryption(
                                                        ciphertext,
                                                    )),
                                                    None,
                                                ),
                                                Err(e) => (None, Some(e.to_string())),
                                            }
                                        }
                                        Request::Nip44Decrypt {
                                            public_key,
                                            ciphertext,
                                        } => {
                                            match nip44::decrypt(
                                                self.keys.secret_key()?,
                                                &public_key,
                                                ciphertext,
                                            ) {
                                                Ok(ciphertext) => (
                                                    Some(ResponseResult::EncryptionDecryption(
                                                        ciphertext,
                                                    )),
                                                    None,
                                                ),
                                                Err(e) => (None, Some(e.to_string())),
                                            }
                                        }
                                        Request::SignEvent(unsigned) => {
                                            match unsigned.sign(&self.keys) {
                                                Ok(event) => (
                                                    Some(ResponseResult::SignEvent(Box::new(
                                                        event,
                                                    ))),
                                                    None,
                                                ),
                                                Err(e) => (None, Some(e.to_string())),
                                            }
                                        }
                                        Request::Ping => (Some(ResponseResult::Pong), None),
                                    }
                                } else {
                                    (None, Some(String::from("Rejected")))
                                };

                                // Compose message
                                let msg: Message = Message::response(id, result, error);

                                // Compose and publish event
                                let event =
                                    EventBuilder::nostr_connect(&self.keys, event.author(), msg)?
                                        .to_event(&self.keys)?;
                                self.pool.send_event(event, RelaySendOptions::new()).await?;
                            }
                        } else {
                            eprintln!("Impossible to decrypt NIP46 message");
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
    fn approve(&self, req: &Request) -> bool;
}
