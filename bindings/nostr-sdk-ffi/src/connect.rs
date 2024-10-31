// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_connect::{client, signer};
use nostr_ffi::nips::nip46::{Nip46Request, NostrConnectURI};
use nostr_ffi::signer::NostrSigner;
use nostr_ffi::{Event, Keys, NostrError, PublicKey, SecretKey, UnsignedEvent};
use nostr_sdk::nostr::nips::nip46::Request;
use nostr_sdk::signer::NostrSigner as _;
use uniffi::Object;

use crate::error::Result;
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
        self.inner
            .relays()
            .into_iter()
            .map(|u| u.to_string())
            .collect()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> Result<NostrConnectURI> {
        Ok(self.inner.bunker_uri().await?.into())
    }
}

#[uniffi::export]
#[async_trait::async_trait]
impl NostrSigner for NostrConnect {
    async fn get_public_key(&self) -> Result<Option<Arc<PublicKey>>, NostrError> {
        Ok(Some(Arc::new(self.inner.get_public_key().await?.into())))
    }

    async fn sign_event(
        &self,
        unsigned: Arc<UnsignedEvent>,
    ) -> Result<Option<Arc<Event>>, NostrError> {
        Ok(Some(Arc::new(
            self.inner
                .sign_event(unsigned.as_ref().deref().clone())
                .await?
                .into(),
        )))
    }

    async fn nip04_encrypt(
        &self,
        public_key: Arc<PublicKey>,
        content: String,
    ) -> Result<String, NostrError> {
        Ok(self
            .inner
            .nip04_encrypt(public_key.as_ref().deref(), &content)
            .await?)
    }

    async fn nip04_decrypt(
        &self,
        public_key: Arc<PublicKey>,
        encrypted_content: String,
    ) -> Result<String, NostrError> {
        Ok(self
            .inner
            .nip04_decrypt(public_key.as_ref().deref(), &encrypted_content)
            .await?)
    }

    async fn nip44_encrypt(
        &self,
        public_key: Arc<PublicKey>,
        content: String,
    ) -> Result<String, NostrError> {
        Ok(self
            .inner
            .nip44_encrypt(public_key.as_ref().deref(), &content)
            .await?)
    }

    async fn nip44_decrypt(
        &self,
        public_key: Arc<PublicKey>,
        payload: String,
    ) -> Result<String, NostrError> {
        Ok(self
            .inner
            .nip44_decrypt(public_key.as_ref().deref(), &payload)
            .await?)
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
    // TODO: change again to `new` (currently python not support async constructor)
    #[uniffi::constructor(default(secret = None, opts = None))]
    pub async fn init(
        secret_key: &SecretKey,
        relays: Vec<String>,
        secret: Option<String>,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: signer::NostrConnectRemoteSigner::new(
                secret_key.deref().clone(),
                relays,
                secret,
                opts.map(|o| o.as_ref().deref().clone()),
            )
            .await?,
        })
    }

    /// Construct remote signer from client URI (`nostrconnect://..`)
    #[uniffi::constructor(default(secret = None, opts = None))]
    pub async fn from_uri(
        uri: &NostrConnectURI,
        secret_key: &SecretKey,
        secret: Option<String>,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: signer::NostrConnectRemoteSigner::from_uri(
                uri.deref().clone(),
                secret_key.deref().clone(),
                secret,
                opts.map(|o| o.as_ref().deref().clone()),
            )
            .await?,
        })
    }

    /// Get signer relays
    pub async fn relays(&self) -> Vec<String> {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|r| r.to_string())
            .collect()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> NostrConnectURI {
        self.inner.bunker_uri().await.into()
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
