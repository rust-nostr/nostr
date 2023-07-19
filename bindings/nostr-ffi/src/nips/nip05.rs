// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip05;

use crate::error::Result;
use crate::{Profile, PublicKey};

pub fn verify_nip05(
    public_key: Arc<PublicKey>,
    nip05: String,
    proxy: Option<String>,
) -> Result<()> {
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(nip05::verify_blocking(
        *public_key.as_ref().deref(),
        nip05,
        proxy,
    )?)
}

pub fn get_nip05_profile(nip05: String, proxy: Option<String>) -> Result<Arc<Profile>> {
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(Arc::new(nip05::get_profile_blocking(nip05, proxy)?.into()))
}
