// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;

use nostr::nips::nip96;
use nostr::Url;
use uniffi::Object;

use crate::error::Result;
use crate::protocol::signer::NostrSigner;

#[derive(Object)]
pub struct ServerConfig {
    inner: nip96::ServerConfig,
}

impl From<nip96::ServerConfig> for ServerConfig {
    fn from(inner: nip96::ServerConfig) -> Self {
        Self { inner }
    }
}

/// Get the nip96.json file on the server and return the JSON as a `ServerConfig`
///
/// <https://github.com/nostr-protocol/nips/blob/master/96.md>
#[uniffi::export(async_runtime = "tokio", default(proxy = None))]
pub async fn get_nip96_server_config(
    server_url: &str,
    proxy: Option<String>,
) -> Result<ServerConfig> {
    let server_url: Url = Url::parse(server_url)?;
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(nip96::get_server_config(server_url, proxy).await?.into())
}

/// Uploads some data to a NIP-96 server and returns the file's download URL
///
/// <https://github.com/nostr-protocol/nips/blob/master/96.md>
#[uniffi::export(async_runtime = "tokio", default(mime_type = None, proxy = None))]
pub async fn nip96_upload(
    signer: &NostrSigner,
    config: &ServerConfig,
    file_data: Vec<u8>,
    mime_type: Option<String>,
    proxy: Option<String>,
) -> Result<String> {
    let proxy: Option<SocketAddr> = match proxy {
        Some(proxy) => Some(proxy.parse()?),
        None => None,
    };
    Ok(nip96::upload_data(
        signer.deref(),
        &config.inner,
        file_data,
        mime_type.as_deref(),
        proxy,
    )
    .await?
    .to_string())
}
