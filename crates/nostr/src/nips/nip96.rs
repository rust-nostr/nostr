// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP96: HTTP File Storage Integration
//!
//! <https://github.com/nostr-protocol/nips/blob/master/96.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use hashes::Hash;
use hashes::sha256::Hash as Sha256Hash;
use serde::{Deserialize, Serialize};

use crate::nips::nip98;
use crate::nips::nip98::{HttpData, HttpMethod};
use crate::types::Url;
use crate::{JsonUtil, NostrSigner, TagKind, TagStandard, Tags};

/// NIP96 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// NIP98 error
    NIP98(nip98::Error),
    /// Invalid URL
    InvalidURL,
    /// Response decode error
    ResponseDecodeError,
    /// Upload error,
    UploadError(String),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP98(e) => e.fmt(f),
            Self::InvalidURL => f.write_str("Invalid URL"),
            Self::ResponseDecodeError => f.write_str("Response decoding error"),
            Self::UploadError(e) => f.write_str(e),
        }
    }
}

impl From<nip98::Error> for Error {
    fn from(e: nip98::Error) -> Self {
        Self::NIP98(e)
    }
}

/// The structure contained in the nip96.json file on nip96 servers
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl JsonUtil for ServerConfig {
    type Err = serde_json::Error;
}

/// NIP-94 event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nip94Event {
    /// Tags
    pub tags: Tags,
}

/// Response status to NIP-96 request
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UploadResponseStatus {
    /// Success
    Success,
    /// Error
    Error,
}

impl UploadResponseStatus {
    /// Check if is success
    #[inline]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

/// Response to a NIP-96 upload request
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

/// Methods and JsonUtil implementation for UploadResponse
impl UploadResponse {
    /// Extract the download URL from the upload response
    ///
    /// Returns an error if the upload was unsuccessful or if the URL cannot be found
    pub fn download_url(&self) -> Result<&Url, Error> {
        if !self.status.is_success() {
            return Err(Error::UploadError(self.message.clone()));
        }

        let nip94_event: &Nip94Event = self
            .nip94_event
            .as_ref()
            .ok_or(Error::ResponseDecodeError)?;
        match nip94_event.tags.find_standardized(TagKind::Url) {
            Some(TagStandard::Url(url)) => Ok(url),
            _ => Err(Error::ResponseDecodeError),
        }
    }
}

impl JsonUtil for UploadResponse {
    type Err = serde_json::Error;
}

/// NIP96 upload request information
/// Contains all data needed to make a file upload request
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UploadRequest {
    /// The URL to POST to
    pub url: Url,
    /// The Authorization header value (NIP98)
    pub authorization: String,
}

impl UploadRequest {
    /// Prepare upload request data
    /// This function prepares the authorization header and returns all the data
    /// needed to make an upload request with the HTTP client.
    /// Note: please create the multipart form data yourself using your
    /// preferred HTTP client's multipart impl.
    pub async fn new<T>(signer: &T, config: &ServerConfig, file_data: &[u8]) -> Result<Self, Error>
    where
        T: NostrSigner,
    {
        let payload: Sha256Hash = Sha256Hash::hash(file_data);
        let data: HttpData =
            HttpData::new(config.api_url.clone(), HttpMethod::POST).payload(payload);
        let authorization: String = data.to_authorization(signer).await?;

        Ok(Self {
            url: config.api_url.clone(),
            authorization,
        })
    }

    /// Get the URL to POST to
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the Authorization header value
    pub fn authorization(&self) -> &str {
        &self.authorization
    }
}

/// Get the NIP96 server config URL for a given server
/// Returns the URL that should be fetched for configuration of the server
pub fn get_server_config_url(server_url: &Url) -> Result<Url, Error> {
    let json_url = server_url
        .join("/.well-known/nostr/nip96.json")
        .map_err(|_| Error::InvalidURL)?;
    Ok(json_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_server_config_url() {
        let server_url = Url::parse("https://nostr.media").unwrap();
        let config_url = get_server_config_url(&server_url).unwrap();
        assert_eq!(
            config_url.to_string(),
            "https://nostr.media/.well-known/nostr/nip96.json"
        );
    }

    #[test]
    fn test_server_config_from_json() {
        let json_response = r#"{
            "api_url": "https://nostr.media/api/v1/nip96/upload",
            "download_url": "https://nostr.media"
        }"#;

        let config = ServerConfig::from_json(json_response).unwrap();
        assert_eq!(
            config.api_url.to_string(),
            "https://nostr.media/api/v1/nip96/upload"
        );
        assert_eq!(config.download_url.to_string(), "https://nostr.media/");
    }

    #[test]
    fn test_upload_response_download_url() {
        let success_response = r#"{
            "status": "success",
            "message": "Upload successful",
            "nip94_event": {
                "tags": [["url", "https://nostr.media/file123.png"]]
            }
        }"#;

        let response = UploadResponse::from_json(success_response).unwrap();
        let url = response.download_url().unwrap();
        assert_eq!(url.to_string(), "https://nostr.media/file123.png");

        let error_response = r#"{
            "status": "error",
            "message": "File too large"
        }"#;

        let response = UploadResponse::from_json(error_response).unwrap();
        let result = response.download_url();
        assert!(result.is_err());
        if let Err(Error::UploadError(msg)) = result {
            assert_eq!(msg, "File too large");
        }
    }
}
