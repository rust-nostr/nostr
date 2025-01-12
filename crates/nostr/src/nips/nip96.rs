// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP96: HTTP File Storage Integration
//!
//! <https://github.com/nostr-protocol/nips/blob/master/96.md>

use core::fmt;

use base64::engine::{general_purpose, Engine};
use hashes::sha256::Hash as Sha256Hash;
use hashes::Hash;
use reqwest::{multipart, Client};
use serde::Deserialize;

use crate::nips::nip98::{HttpData, HttpMethod};
use crate::types::Url;
use crate::util::JsonUtil;
use crate::{EventBuilder, NostrSigner, TagKind, TagStandard, Tags};

/// NIP96 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP-98 auth event sign error
    AuthEventSignError,
    /// Invalid URL
    InvalidURL,
    /// Response decode error
    ResponseDecodeError,
    /// Multipart MIME error
    MultipartMimeError,
    /// Fetch error
    ClientFetchError,
    /// Upload error,
    UploadError,
    /// Server descriptor fetch error
    CannotFetchDescriptor,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidURL => write!(f, "Invalid URL"),
            Self::ClientFetchError => write!(f, "Client fetch error"),
            Self::ResponseDecodeError => write!(f, "Response decoding error"),
            Self::MultipartMimeError => write!(f, "Invalid MIME type for the multipart form"),
            Self::UploadError => write!(f, "File upload error"),
            Self::CannotFetchDescriptor => {
                write!(f, "Cannot fetch nip96.json file from server")
            }
            Self::AuthEventSignError => write!(f, "Failed to sign NIP98 auth event"),
        }
    }
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

#[derive(Debug, Deserialize)]
struct Nip94Event {
    tags: Tags,
}

/// Response to a NIP-96 upload request
#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    nip94_event: Nip94Event,
}

/// Build the NIP98 base64-encoded HTTP Authorization header
async fn build_nip98_auth_event<T>(signer: &T, data: HttpData) -> Result<String, Error>
where
    T: NostrSigner,
{
    let event = EventBuilder::http_auth(data)
        .sign(signer)
        .await
        .map_err(|_| Error::AuthEventSignError)?;
    Ok(general_purpose::STANDARD.encode(event.as_json()))
}

/// Get the nip96.json file on the server and return the JSON as a [`ServerConfig`]
pub async fn get_server_config(server_url: Url) -> Result<ServerConfig, Error> {
    let json_url = server_url
        .join("/.well-known/nostr/nip96.json")
        .map_err(|_| Error::InvalidURL)?;

    let response = Client::new()
        .get(json_url)
        .send()
        .await
        .map_err(|_| Error::ClientFetchError)?;

    response
        .json()
        .await
        .map_err(|_| Error::CannotFetchDescriptor)
}

/// Uploads some data to a NIP-96 server and returns the file's download URL
pub async fn upload_data<T>(
    signer: &T,
    server_url: Url,
    file_data: Vec<u8>,
    mime_type: Option<&str>,
) -> Result<Url, Error>
where
    T: NostrSigner,
{
    // Get server config
    let desc: ServerConfig = get_server_config(server_url).await?;

    // Build NIP98 Authorization header
    let payload: Sha256Hash = Sha256Hash::hash(&file_data);
    let data: HttpData = HttpData::new(desc.api_url.clone(), HttpMethod::POST).payload(payload);
    let nip98_auth: String = build_nip98_auth_event(signer, data).await?;

    let form_file_part = multipart::Part::bytes(file_data).file_name("filename");

    // Set the part's MIME type, or leave it as is if mime_type is None
    let part = match mime_type {
        Some(mime) => form_file_part
            .mime_str(mime)
            .map_err(|_| Error::MultipartMimeError)?,
        None => form_file_part,
    };

    let response = Client::new()
        .post(desc.api_url)
        .header("Authorization", format!("Nostr {}", nip98_auth).as_str())
        .multipart(multipart::Form::new().part("file", part))
        .send()
        .await
        .map_err(|_| Error::UploadError)?;

    // Decode response
    let res: UploadResponse = response
        .json()
        .await
        .map_err(|_| Error::ResponseDecodeError)?;

    // Extract file url
    match res.nip94_event.tags.find_standardized(TagKind::Url) {
        Some(TagStandard::Url(url)) => Ok(url.clone()),
        _ => Err(Error::ResponseDecodeError),
    }
}
