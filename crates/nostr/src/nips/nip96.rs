// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP96: HTTP File Storage Integration
//!
//! <https://github.com/nostr-protocol/nips/blob/master/96.md>

use alloc::string::{String, ToString};
use core::fmt;
use std::net::SocketAddr;

use hashes::sha256::Hash as Sha256Hash;
use hashes::Hash;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use reqwest::{multipart, Client, Response};
use serde::{Deserialize, Serialize};

use crate::nips::nip98;
use crate::nips::nip98::{HttpData, HttpMethod};
use crate::types::Url;
use crate::{NostrSigner, TagKind, TagStandard, Tags};

/// NIP96 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Reqwest error
    Reqwest(String),
    /// NIP98 error
    NIP98(nip98::Error),
    /// Invalid URL
    InvalidURL,
    /// Response decode error
    ResponseDecodeError,
    /// Multipart MIME error
    MultipartMimeError,
    /// Upload error,
    UploadError(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "{e}"),
            Self::NIP98(e) => write!(f, "{e}"),
            Self::InvalidURL => write!(f, "Invalid URL"),
            Self::ResponseDecodeError => write!(f, "Response decoding error"),
            Self::MultipartMimeError => write!(f, "Invalid MIME type for the multipart form"),
            Self::UploadError(e) => write!(f, "File upload error: {e}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e.to_string())
    }
}

impl From<nip98::Error> for Error {
    fn from(e: nip98::Error) -> Self {
        Self::NIP98(e)
    }
}

fn make_client(_proxy: Option<SocketAddr>) -> Result<Client, Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let client: Client = {
        let mut builder = Client::builder();
        if let Some(proxy) = _proxy {
            let proxy = format!("socks5h://{proxy}");
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        builder.build()?
    };

    #[cfg(target_arch = "wasm32")]
    let client: Client = Client::new();

    Ok(client)
}

/// The structure contained in the nip96.json file on nip96 servers
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct ServerConfig {
    /// API URL
    pub api_url: Url,
    /// Download URL
    pub download_url: Url,
    /// Delegated URL
    pub delegated_to_url: Option<Url>,
    /// Allowed content types
    pub content_types: Option<Vec<String>>,
}

/// NIP-94 event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nip94Event {
    /// Tags
    pub tags: Tags,
}

/// Response status to NIP-96 request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UploadResponseStatus {
    /// Success
    Success,
    /// Error
    Error,
}

/// Response to a NIP-96 upload request
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    /// Status
    pub status: UploadResponseStatus,
    /// Free text success, failure or info message
    pub message: String,
    /// NIP-94 event
    ///
    /// `nip94_event` field is absent if unsuccessful upload
    pub nip94_event: Option<Nip94Event>,
}

/// Get the nip96.json file on the server and return the JSON as a [`ServerConfig`]
///
/// **Proxy is ignored for WASM targets!**
pub async fn get_server_config(
    server_url: Url,
    proxy: Option<SocketAddr>,
) -> Result<ServerConfig, Error> {
    let json_url = server_url
        .join("/.well-known/nostr/nip96.json")
        .map_err(|_| Error::InvalidURL)?;

    let client: Client = make_client(proxy)?;

    let response = client.get(json_url).send().await?;

    Ok(response.json().await?)
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
            .map_err(|_| Error::MultipartMimeError)?,
        None => form_file_part,
    };

    Ok(multipart::Form::new().part("file", part))
}

/// Uploads some data to a NIP-96 server and returns the file's download URL
///
/// **Proxy is ignored for WASM targets!**
pub async fn upload_data<T>(
    signer: &T,
    desc: &ServerConfig,
    file_data: Vec<u8>,
    mime_type: Option<&str>,
    proxy: Option<SocketAddr>,
) -> Result<Url, Error>
where
    T: NostrSigner,
{
    // Build NIP98 Authorization header
    let payload: Sha256Hash = Sha256Hash::hash(&file_data);
    let data: HttpData = HttpData::new(desc.api_url.clone(), HttpMethod::POST).payload(payload);
    let nip98_auth: String = data.to_authorization(signer).await?;

    // Make form
    let form: multipart::Form = make_multipart_form(file_data, mime_type)?;

    // Make client
    let client: Client = make_client(proxy)?;

    // Send
    let response: Response = client
        .post(desc.api_url.clone())
        .header("Authorization", nip98_auth)
        .multipart(form)
        .send()
        .await?;

    // Decode response
    let res: UploadResponse = response.json().await?;

    // Check status
    if res.status == UploadResponseStatus::Error {
        return Err(Error::UploadError(res.message));
    }

    // Extract url
    let nip94_event: Nip94Event = res.nip94_event.ok_or(Error::ResponseDecodeError)?;
    match nip94_event.tags.find_standardized(TagKind::Url) {
        Some(TagStandard::Url(url)) => Ok(url.clone()),
        _ => Err(Error::ResponseDecodeError),
    }
}
