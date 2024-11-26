// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr::NostrSigner;
use nostr_connect::{client, signer};
use nostr_sdk::nostr::nips::nip46::Request;
use uniffi::{Object, Record};

use crate::error::Result;
use crate::protocol::nips::nip46::{Nip46Request, NostrConnectURI};
use crate::protocol::{Event, Keys, PublicKey, UnsignedEvent};
use crate::relay::RelayOptions;

#[derive(Object)]
pub struct NostrConnect {
    inner: client::NostrConnect,
}

impl Deref for NostrConnect {
    type Target = client::NostrConnect;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<client::NostrConnect> for NostrConnect {
    fn from(inner: client::NostrConnect) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NostrConnect {
    /// Construct Nostr Connect client
    #[uniffi::constructor]
    pub fn new(
        uri: &NostrConnectURI,
        app_keys: &Keys,
        timeout: Duration,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: client::NostrConnect::new(
                uri.deref().clone(),
                app_keys.deref().clone(),
                timeout,
                opts.map(|k| k.as_ref().deref().clone()),
            )?,
        })
    }

    /// Get signer relays
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays().iter().map(|u| u.to_string()).collect()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> Result<NostrConnectURI> {
        Ok(self.inner.bunker_uri().await?.into())
    }

    pub async fn get_public_key(&self) -> Result<PublicKey> {
        Ok(self.inner.get_public_key().await?.into())
    }

    pub async fn sign_event(&self, unsigned_event: &UnsignedEvent) -> Result<Event> {
        Ok(self
            .inner
            .sign_event(unsigned_event.deref().clone())
            .await?
            .into())
    }

    pub async fn nip04_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String> {
        Ok(self
            .inner
            .nip04_encrypt(public_key.deref(), content)
            .await?)
    }

    pub async fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: &str,
    ) -> Result<String> {
        Ok(self
            .inner
            .nip04_decrypt(public_key.deref(), encrypted_content)
            .await?)
    }

    pub async fn nip44_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String> {
        Ok(self
            .inner
            .nip44_encrypt(public_key.deref(), content)
            .await?)
    }

    pub async fn nip44_decrypt(&self, public_key: &PublicKey, payload: &str) -> Result<String> {
        Ok(self
            .inner
            .nip44_decrypt(public_key.deref(), payload)
            .await?)
    }
}

#[derive(Record)]
pub struct NostrConnectKeys {
    pub signer: Arc<Keys>,
    pub user: Arc<Keys>,
}

impl From<NostrConnectKeys> for signer::NostrConnectKeys {
    fn from(keys: NostrConnectKeys) -> Self {
        Self {
            signer: keys.signer.as_ref().deref().clone(),
            user: keys.user.as_ref().deref().clone(),
        }
    }
}

/// Nostr Connect Signer
///
/// Signer that listen for requests from client, handle them and send the response.
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Object)]
pub struct NostrConnectRemoteSigner {
    inner: signer::NostrConnectRemoteSigner,
}

#[uniffi::export(async_runtime = "tokio")]
impl NostrConnectRemoteSigner {
    #[uniffi::constructor(default(secret = None, opts = None))]
    pub fn new(
        keys: NostrConnectKeys,
        relays: Vec<String>,
        secret: Option<String>,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: signer::NostrConnectRemoteSigner::new(
                keys.into(),
                relays,
                secret,
                opts.map(|o| o.as_ref().deref().clone()),
            )?,
        })
    }

    /// Construct remote signer from client URI (`nostrconnect://..`)
    #[uniffi::constructor(default(secret = None, opts = None))]
    pub fn from_uri(
        uri: &NostrConnectURI,
        keys: NostrConnectKeys,
        secret: Option<String>,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: signer::NostrConnectRemoteSigner::from_uri(
                uri.deref().clone(),
                keys.into(),
                secret,
                opts.map(|o| o.as_ref().deref().clone()),
            )?,
        })
    }

    /// Get signer relays
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays().iter().map(|r| r.to_string()).collect()
    }

    /// Get `bunker` URI
    pub fn bunker_uri(&self) -> NostrConnectURI {
        self.inner.bunker_uri().into()
    }

    /// Serve signer
    pub async fn serve(&self, actions: Arc<dyn NostrConnectSignerActions>) -> Result<()> {
        let actions = FFINostrConnectSignerActions(actions);
        Ok(self.inner.serve(actions).await?)
    }
}

struct FFINostrConnectSignerActions(Arc<dyn NostrConnectSignerActions>);

impl signer::NostrConnectSignerActions for FFINostrConnectSignerActions {
    fn approve(&self, req: &Request) -> bool {
        self.0.approve(req.to_owned().into())
    }
}

#[uniffi::export(with_foreign)]
pub trait NostrConnectSignerActions: Send + Sync {
    /// Approve
    fn approve(&self, req: Nip46Request) -> bool;
}
