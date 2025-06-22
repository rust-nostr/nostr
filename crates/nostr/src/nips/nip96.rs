// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP96: HTTP File Storage Integration
//!
//! <https://github.com/nostr-protocol/nips/blob/master/96.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use hashes::sha256::Hash as Sha256Hash;
use hashes::Hash;
use serde::{Deserialize, Serialize};

use crate::nips::nip98;
use crate::nips::nip98::{HttpData, HttpMethod};
use crate::types::Url;
use crate::{NostrSigner, TagKind, TagStandard, Tags};

/// NIP96 error
#[derive(Debug)]
pub enum Error {
    /// NIP98 error
    NIP98(nip98::Error),
    /// Invalid URL
    InvalidURL,
    /// Response decode error
    ResponseDecodeError,
    /// Upload error,
    UploadError(String),
    /// JSON parsing error
    Json(serde_json::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP98(e) => write!(f, "{e}"),
            Self::InvalidURL => write!(f, "Invalid URL"),
            Self::ResponseDecodeError => write!(f, "Response decoding error"),
            Self::UploadError(e) => write!(f, "File upload error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl From<nip98::Error> for Error {
    fn from(e: nip98::Error) -> Self {
        Self::NIP98(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
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

/// NIP96 server config request information
///
/// Contains the URL needed to fetch server configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerConfigRequest {
    /// The URL to fetch
    pub url: String,
}

impl ServerConfigRequest {
    /// Create a new server config request
    pub fn new(server_url: Url) -> Result<Self, Error> {
        let config_url = get_server_config_url(&server_url)?;
        Ok(Self { url: config_url })
    }

    /// Get the URL to fetch
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// NIP96 upload request information
///
/// Contains all data needed to make a file upload request
#[derive(Debug, Clone)]
pub struct UploadRequest {
    /// The URL to POST to
    pub url: String,
    /// The Authorization header value (NIP98)
    pub authorization: String,
}

impl UploadRequest {
    /// Prepare upload request data
    ///
    /// This function prepares the authorization header and returns all the data
    /// needed to make an upload request with the HTTP client.
    ///
    /// Note: please create the multipart form data yourself using your
    /// preferred HTTP client's multipart impl.
    ///
    pub async fn new<T>(signer: &T, config: &ServerConfig, file_data: &[u8]) -> Result<Self, Error>
    where
        T: NostrSigner,
    {
        let payload: Sha256Hash = Sha256Hash::hash(file_data);
        let data: HttpData =
            HttpData::new(config.api_url.clone(), HttpMethod::POST).payload(payload);
        let authorization: String = data.to_authorization(signer).await?;

        Ok(Self {
            url: config.api_url.to_string(),
            authorization,
        })
    }

    /// Get the URL to POST to
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the Authorization header value
    pub fn authorization(&self) -> &str {
        &self.authorization
    }
}

/// Get the NIP96 server config URL for a given server
///
/// Returns the URL that should be fetched for configuration of the server
pub fn get_server_config_url(server_url: &Url) -> Result<String, Error> {
    let json_url = server_url
        .join("/.well-known/nostr/nip96.json")
        .map_err(|_| Error::InvalidURL)?;
    Ok(json_url.to_string())
}

/// Parse the server config from JSON data
///
/// This function would allows you to parse server config without any HTTP dependencies.
/// Fetch the JSON data using your preferred HTTP client and pass it here.
///
pub fn server_config_from_response(json_response: &str) -> Result<ServerConfig, Error> {
    let config: ServerConfig = serde_json::from_str(json_response)?;
    Ok(config)
}

/// Parse upload response and extract download URL
///
/// This function would extracts the download URL from a NIP96 upload response
/// Use this after you've made the upload request with your HTTP client.
///
pub fn upload_response_to_url(json_response: &str) -> Result<Url, Error> {
    let res: UploadResponse = serde_json::from_str(json_response)?;
    if res.status == UploadResponseStatus::Error {
        return Err(Error::UploadError(res.message));
    }
    let nip94_event: Nip94Event = res.nip94_event.ok_or(Error::ResponseDecodeError)?;
    match nip94_event.tags.find_standardized(TagKind::Url) {
        Some(TagStandard::Url(url)) => Ok(url.clone()),
        _ => Err(Error::ResponseDecodeError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_request() {
        let server_url = Url::parse("https://nostr.media").unwrap();
        let request = ServerConfigRequest::new(server_url).unwrap();

        assert_eq!(
            request.url(),
            "https://nostr.media/.well-known/nostr/nip96.json"
        );
    }

    #[test]
    fn test_get_server_config_url() {
        let server_url = Url::parse("https://nostr.media").unwrap();
        let config_url = get_server_config_url(&server_url).unwrap();
        assert_eq!(
            config_url,
            "https://nostr.media/.well-known/nostr/nip96.json"
        );
    }

    #[test]
    fn test_server_config_from_response() {
        let json_response = r#"{
            "api_url": "https://nostr.media/api/v1/nip96/upload",
            "download_url": "https://nostr.media"
        }"#;

        let config = server_config_from_response(json_response).unwrap();
        assert_eq!(
            config.api_url.to_string(),
            "https://nostr.media/api/v1/nip96/upload"
        );
        assert_eq!(config.download_url.to_string(), "https://nostr.media/");
    }

    #[test]
    fn test_upload_response_to_url() {
        let success_response = r#"{
            "status": "success",
            "message": "Upload successful",
            "nip94_event": {
                "tags": [["url", "https://nostr.media/file123.png"]]
            }
        }"#;

        let url = upload_response_to_url(success_response).unwrap();
        assert_eq!(url.to_string(), "https://nostr.media/file123.png");

        let error_response = r#"{
            "status": "error",
            "message": "File too large"
        }"#;

        let result = upload_response_to_url(error_response);
        assert!(result.is_err());
        if let Err(Error::UploadError(msg)) = result {
            assert_eq!(msg, "File too large");
        }
    }
}
