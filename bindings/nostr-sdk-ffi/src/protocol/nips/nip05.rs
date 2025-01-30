// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;

use nostr::nips::nip05;
use uniffi::Object;

use crate::error::Result;
use crate::protocol::key::PublicKey;

#[derive(Object)]
pub struct Nip05Profile {
    inner: nip05::Nip05Profile,
}

impl From<nip05::Nip05Profile> for Nip05Profile {
    fn from(inner: nip05::Nip05Profile) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip05Profile {
    pub fn public_key(&self) -> PublicKey {
        self.inner.public_key.into()
    }

    /// Get relays
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.iter().map(|u| u.to_string()).collect()
    }

    /// Get NIP46 relays
    pub fn nip46(&self) -> Vec<String> {
        self.inner.nip46.iter().map(|u| u.to_string()).collect()
    }
}

#[uniffi::export(async_runtime = "tokio", default(proxy = None))]
pub async fn verify_nip05(
    public_key: &PublicKey,
    nip05: &str,
    proxy: Option<String>,
) -> Result<bool> {
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(nip05::verify(public_key.deref(), nip05, proxy).await?)
}

/// Get NIP05 profile
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
#[uniffi::export(async_runtime = "tokio", default(proxy = None))]
pub async fn get_nip05_profile(nip05: &str, proxy: Option<String>) -> Result<Nip05Profile> {
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(nip05::profile(nip05, proxy).await?.into())
}
