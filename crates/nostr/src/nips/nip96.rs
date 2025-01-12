//! NIP96: HTTP File Storage Integration
//!
//! <https://github.com/nostr-protocol/nips/blob/master/96.md>

use core::fmt;
use serde::{Deserialize, Serialize};

use reqwest::{multipart, Client};

use hashes::sha256::Hash as Sha256Hash;
use hashes::Hash;

use base64::engine::{general_purpose, Engine};

use crate::nips::nip98::{HttpData, HttpMethod};
use crate::types::Url;
use crate::util::JsonUtil;
use crate::{EventBuilder, NostrSigner};

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
    CannotFetchDescriptor(Url),
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
            Self::CannotFetchDescriptor(url) => {
                write!(f, "Cannot fetch nip96.json file from server: {}", url)
            }
            Self::AuthEventSignError => write!(f, "Failed to sign NIP98 auth event"),
        }
    }
}

/// The structure contained in the nip96.json file on nip96 servers
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// API URL
    pub api_url: String,
    /// Download URL
    pub download_url: String,
    /// Delegated URL
    pub delegated_to_url: Option<String>,
    /// Allowed content types
    pub content_types: Option<Vec<String>>,
}

/// NIP-94 event
#[derive(Debug, Serialize, Deserialize)]
pub struct Nip94Event {
    tags: Vec<Vec<String>>,
}

/// Response to a NIP-96 upload request
#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    nip94_event: Nip94Event,
}

/// Build the base64-encoded HTTP Authorization header
pub async fn build_nip98_auth_event<T>(
    signer: &T,
    url: Url,
    hash: Sha256Hash,
    method: HttpMethod,
) -> Result<String, Error>
where
    T: NostrSigner,
{
    let data = HttpData::new(url.clone(), method).payload(hash);
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
        .get(json_url.clone())
        .send()
        .await
        .map_err(|_| Error::ClientFetchError)?;

    if let Ok(config) = response.json::<ServerConfig>().await {
        Ok(config)
    } else {
        Err(Error::CannotFetchDescriptor(json_url.clone()))
    }
}

/// Returns the API endpoint URL for a NIP-96 server's URL
pub async fn get_api_url_for(server_url: Url) -> Result<Url, Error> {
    if let Ok(desc) = get_server_config(server_url.clone()).await {
        if let Ok(url) = Url::parse(desc.api_url.as_str()) {
            Ok(url)
        } else {
            Err(Error::InvalidURL)
        }
    } else {
        Err(Error::CannotFetchDescriptor(server_url))
    }
}

/// Uploads some data to a NIP-96 server and returns the file's download URL
pub async fn upload_data<T>(
    server_url: Url,
    data: Vec<u8>,
    mime_type: Option<&str>,
    signer: &T,
) -> Result<Url, Error>
where
    T: NostrSigner,
{
    let Ok(api_url) = get_api_url_for(server_url.clone()).await else {
        return Err(Error::CannotFetchDescriptor(server_url));
    };

    let payload = Sha256Hash::hash(&data[..]);

    let nip98_auth =
        build_nip98_auth_event(signer, api_url.clone(), payload.clone(), HttpMethod::POST).await?;

    let form_file_part = multipart::Part::bytes(data).file_name("filename");

    // Set the part's MIME type, or leave it as is if mime_type is None
    let part = match mime_type {
        Some(mime) => form_file_part
            .mime_str(mime)
            .map_err(|_| Error::MultipartMimeError)?,
        None => form_file_part,
    };

    let response = Client::new()
        .post(api_url)
        .header("Authorization", format!("Nostr {}", nip98_auth).as_str())
        .multipart(multipart::Form::new().part("file", part))
        .send()
        .await
        .map_err(|_| Error::UploadError)?;

    if let Ok(resp) = response.json::<UploadResponse>().await {
        for tag in resp.nip94_event.tags.iter() {
            match tag[0].as_str() {
                "url" => {
                    return Ok(Url::parse(tag[1].as_str()).unwrap());
                }
                _ => continue,
            }
        }

        Err(Error::UploadError)
    } else {
        Err(Error::ResponseDecodeError)
    }
}
