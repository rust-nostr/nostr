// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::nips::nip46::{Nip46Request, NostrConnectURI};
use nostr_ffi::{Keys, PublicKey, SecretKey};
use nostr_sdk::nostr::nips::nip46::Request;
use nostr_sdk::signer;
use uniffi::Object;

use crate::error::Result;
use crate::relay::RelayOptions;

#[derive(Object)]
pub struct Nip46Signer {
    inner: signer::Nip46Signer,
}

impl Deref for Nip46Signer {
    type Target = signer::Nip46Signer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<signer::Nip46Signer> for Nip46Signer {
    fn from(inner: signer::Nip46Signer) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Nip46Signer {
    // TODO: change again to `new` (currently python not support async constructor)
    /// Construct Nostr Connect client
    #[uniffi::constructor]
    pub async fn init(
        uri: &NostrConnectURI,
        app_keys: &Keys,
        timeout: Duration,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: signer::Nip46Signer::new(
                uri.deref().clone(),
                app_keys.deref().clone(),
                timeout,
                opts.map(|k| k.as_ref().deref().clone()),
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
            .map(|u| u.to_string())
            .collect()
    }

    /// Get signer public key
    pub fn signer_public_key(&self) -> PublicKey {
        self.inner.signer_public_key().into()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> NostrConnectURI {
        self.inner.bunker_uri().await.into()
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
