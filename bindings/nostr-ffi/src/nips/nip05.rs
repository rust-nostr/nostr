// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip05;

use crate::error::Result;
use crate::nips::nip19::Nip19Profile;
use crate::PublicKey;

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

#[uniffi::export(async_runtime = "tokio", default(proxy = None))]
pub async fn get_nip05_profile(nip05: &str, proxy: Option<String>) -> Result<Arc<Nip19Profile>> {
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(Arc::new(nip05::get_profile(nip05, proxy).await?.into()))
}
