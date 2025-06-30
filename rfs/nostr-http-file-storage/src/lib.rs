// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr HTTP File Storage client (NIP-96)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/96.md>

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

use std::fmt;
#[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
use std::net::SocketAddr;
use std::time::Duration;

use nostr::nips::nip96::{self, ServerConfig, UploadRequest, UploadResponse};
use nostr::signer::NostrSigner;
use nostr::types::url::Url;
#[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
use reqwest::Proxy;
use reqwest::{multipart, Client, ClientBuilder, Response};

pub mod prelude;

/// Nostr HTTP File Storage client error
#[derive(Debug)]
pub enum Error {
    /// Reqwest error
    Reqwest(reqwest::Error),
    /// NIP-96 error
    NIP96(nip96::Error),
    /// Multipart MIME error
    MultipartMime,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "{e}"),
            Self::NIP96(e) => write!(f, "{e}"),
            Self::MultipartMime => write!(f, "Invalid MIME type for the multipart form"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<nip96::Error> for Error {
    fn from(e: nip96::Error) -> Self {
        Self::NIP96(e)
    }
}

/// Nostr HTTP File Storage client
#[derive(Debug, Clone)]
pub struct NostrHttpFileStorageClientBuilder {
    /// Socks5 proxy
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    pub proxy: Option<SocketAddr>,
    /// Timeout
    pub timeout: Duration,
}

impl Default for NostrHttpFileStorageClientBuilder {
    fn default() -> Self {
        Self {
            #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
            proxy: None,
            timeout: Duration::from_secs(60),
        }
    }
}

impl NostrHttpFileStorageClientBuilder {
    /// New default builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set proxy
    #[inline]
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    pub fn proxy(mut self, addr: SocketAddr) -> Self {
        self.proxy = Some(addr);
        self
    }

    /// Set timeout
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the client
    pub fn build(self) -> Result<NostrHttpFileStorageClient, Error> {
        // Construct builder
        let mut builder: ClientBuilder = Client::builder();

        // Set proxy
        #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
        if let Some(proxy) = self.proxy {
            let proxy: String = format!("socks5h://{proxy}");
            builder = builder.proxy(Proxy::all(proxy)?);
        }

        // Set timeout
        builder = builder.timeout(self.timeout);

        // Build client
        let client: Client = builder.build()?;

        // Construct client
        Ok(NostrHttpFileStorageClient::from_client(client))
    }
}

/// Nostr HTTP File Storage client
#[derive(Debug, Clone)]
pub struct NostrHttpFileStorageClient {
    client: Client,
}

impl Default for NostrHttpFileStorageClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NostrHttpFileStorageClient {
    /// Construct a default client
    #[inline]
    pub fn new() -> Self {
        Self::builder().build().expect("Failed to build client")
    }

    /// Construct from reqwest [`Client`].
    #[inline]
    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    /// Get a builder
    #[inline]
    pub fn builder() -> NostrHttpFileStorageClientBuilder {
        NostrHttpFileStorageClientBuilder::default()
    }

    /// Get the nip96.json file on the server and return the JSON as a [`ServerConfig`]
    pub async fn get_server_config(&self, server_url: &Url) -> Result<ServerConfig, Error> {
        let nip96_url: Url = nip96::get_server_config_url(server_url)?;

        let response = self.client.get(nip96_url).send().await?;

        // Deserialize response
        Ok(response.json().await?)
    }

    /// Uploads some data to a NIP-96 server and returns the file's download URL
    pub async fn upload<T>(
        &self,
        signer: &T,
        config: &ServerConfig,
        file_data: Vec<u8>,
        mime_type: Option<&str>,
    ) -> Result<Url, Error>
    where
        T: NostrSigner,
    {
        // Create new request
        let req: UploadRequest = UploadRequest::new(signer, config, &file_data).await?;

        // Make form
        let form: multipart::Form = make_multipart_form(file_data, mime_type)?;

        // Send
        let response: Response = self
            .client
            .post(config.api_url.clone())
            .header("Authorization", req.authorization())
            .multipart(form)
            .send()
            .await?;

        // Decode response
        let res: UploadResponse = response.json().await?;

        // Try to extract download URL
        Ok(res.download_url().cloned()?)
    }
}

fn make_multipart_form(
    file_data: Vec<u8>,
    mime_type: Option<&str>,
) -> Result<multipart::Form, Error> {
    let form_file_part = multipart::Part::bytes(file_data).file_name("filename");

    // Set the part's MIME type, or leave it as is if mime_type is None
    let part = match mime_type {
        Some(mime) => form_file_part
            .mime_str(mime)
            .map_err(|_| Error::MultipartMime)?,
        None => form_file_part,
    };

    Ok(multipart::Form::new().part("file", part))
}
