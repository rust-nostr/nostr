use std::error::Error;
use std::time::Duration;

use base64::Engine;
use nostr::hashes::{sha256, Hash};
use nostr::signer::NostrSigner;
use nostr::{EventBuilder, PublicKey, Timestamp};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, RANGE};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::redirect::Policy;

use crate::bud01::{
    BlossomAuthorization, BlossomAuthorizationScope, BlossomAuthorizationVerb,
    BlossomBuilderExtension,
};
use crate::bud02::BlobDescriptor;

/// A client for interacting with a Blossom server
///
/// <https://github.com/hzrd149/blossom>
#[derive(Debug, Clone)]
pub struct BlossomClient {
    base_url: String,
    client: reqwest::Client,
}

impl BlossomClient {
    /// Creates a new `BlossomClient` with the given base URL.
    pub fn new<T>(base_url: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            base_url: base_url.into(),
            client: Self::build_client().unwrap(),
        }
    }

    /// Builds the reqwest client
    fn build_client() -> reqwest::Result<reqwest::Client> {
        let builder = reqwest::Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder.redirect(Policy::limited(10));
        builder.build()
    }

    /// Uploads a blob to the Blossom server.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/02.md>
    pub async fn upload_blob<T>(
        &self,
        data: Vec<u8>,
        content_type: Option<String>,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: Option<&T>,
    ) -> Result<BlobDescriptor, Box<dyn Error>>
    where
        T: NostrSigner,
    {
        let hash = sha256::Hash::hash(&data);
        let file_hashes = vec![hash];
        let url = format!("{}/upload", self.base_url);
        let mut request = self.client.put(&url).body(data);
        let mut headers = HeaderMap::new();

        if let Some(ct) = content_type {
            headers.insert(CONTENT_TYPE, ct.parse()?);
        }
        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::Upload,
                "Blossom upload authorization",
                BlossomAuthorizationScope::BlobSha256Hashes(file_hashes),
            )?;
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        request = request.headers(headers);

        let response = request.send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let descriptor: BlobDescriptor = response.json().await?;
                Ok(descriptor)
            }
            _ => Err(Self::extract_error("Failed to upload blob", &response)),
        }
    }

    /// Lists blobs uploaded by a specific pubkey.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/02.md>
    pub async fn list_blobs<T>(
        &self,
        pubkey: &PublicKey,
        since: Option<Timestamp>,
        until: Option<Timestamp>,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: Option<&T>,
    ) -> Result<Vec<BlobDescriptor>, Box<dyn Error>>
    where
        T: NostrSigner,
    {
        let mut url = format!("{}/list/{}", self.base_url, pubkey);
        let mut query_params = Vec::new();
        if let Some(since) = since {
            query_params.push(format!("since=\"{}\"", since));
        }
        if let Some(until) = until {
            query_params.push(format!("until=\"{}\"", until));
        }
        if !query_params.is_empty() {
            url.push_str(&format!("?{}", query_params.join("&")));
        }
        let mut request = self.client.get(&url);
        let mut headers = HeaderMap::new();
        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::List,
                "Blossom list authorization",
                BlossomAuthorizationScope::ServerUrl(self.base_url.clone()),
            )?;
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        request = request.headers(headers);
        let response = request.send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let descriptors: Vec<BlobDescriptor> = response.json().await?;
                Ok(descriptors)
            }
            _ => Err(Self::extract_error("Failed to list blobs", &response)),
        }
    }

    /// Retrieves a blob from the Blossom server, with optional authorization.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    pub async fn get_blob<T>(
        &self,
        sha256: sha256::Hash,
        range: Option<String>,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: Option<&T>,
    ) -> Result<Vec<u8>, Box<dyn Error>>
    where
        T: NostrSigner,
    {
        let url = format!("{}/{}", self.base_url, sha256);
        let mut request = self.client.get(&url);
        let mut headers = HeaderMap::new();

        if let Some(range_value) = range {
            headers.insert(RANGE, HeaderValue::from_str(&range_value)?);
        }
        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::Get,
                "Blossom get authorization",
                BlossomAuthorizationScope::BlobSha256Hashes(vec![sha256.clone()]),
            )?;
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        request = request.headers(headers);

        let response = request.send().await?;

        if response.status().is_redirection() {
            if let Some(location) = response.headers().get("Location") {
                let location_str = location.to_str()?;
                if !location_str.contains(&sha256.to_string()) {
                    return Err("Redirect URL does not contain sha256 hash".into());
                }
            } else {
                return Err("Redirect response missing Location header".into());
            }
        }
        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::PARTIAL_CONTENT => {
                Ok(response.bytes().await?.to_vec())
            }
            _ => Err(Self::extract_error("Failed to get blob", &response)),
        }
    }

    /// Checks if a blob exists on the Blossom server.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    pub async fn has_blob<T>(
        &self,
        sha256: sha256::Hash,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: Option<&T>,
    ) -> Result<bool, Box<dyn Error>>
    where
        T: NostrSigner,
    {
        let url = format!("{}/{}", self.base_url, sha256);
        let mut request = self.client.head(&url);
        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::Get,
                "Blossom get authorization",
                BlossomAuthorizationScope::BlobSha256Hashes(vec![sha256.clone()]),
            )?;
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let mut headers = HeaderMap::new();
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
            request = request.headers(headers);
        }
        let response = request.send().await?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(true),
            reqwest::StatusCode::NOT_FOUND => Ok(false),
            _ => Err(Self::extract_error(
                "Unexpected HTTP status code",
                &response,
            )),
        }
    }

    /// Deletes a blob from the Blossom server.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/02.md>
    pub async fn delete_blob<T>(
        &self,
        sha256: sha256::Hash,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: &T,
    ) -> Result<(), Box<dyn Error>>
    where
        T: NostrSigner,
    {
        let url = format!("{}/{}", self.base_url, sha256);
        let mut headers = HeaderMap::new();
        let default_auth = self.default_auth(
            BlossomAuthorizationVerb::Delete,
            "Blossom delete authorization",
            BlossomAuthorizationScope::BlobSha256Hashes(vec![sha256.clone()]),
        )?;
        let final_auth = authorization_options
            .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
            .unwrap_or(default_auth);
        let auth_header = Self::build_auth_header(signer, &final_auth).await?;
        headers.insert(AUTHORIZATION, auth_header);
        let response = self.client.delete(&url).headers(headers).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Self::extract_error("Failed to delete blob", &response))
        }
    }

    /// Returns a default BlossomAuthorization object based on the parameters provided.
    fn default_auth<T>(
        &self,
        action: BlossomAuthorizationVerb,
        default_content: T,
        default_scope: BlossomAuthorizationScope,
    ) -> Result<BlossomAuthorization, Box<dyn Error>>
    where
        T: Into<String>,
    {
        let expiration_timestamp = Timestamp::now() + Duration::from_secs(300);
        Ok(BlossomAuthorization::new(
            default_content.into(),
            expiration_timestamp,
            action,
            default_scope,
        ))
    }

    /// Updates a default BlossomAuthorization fixture with the provided options.
    pub fn update_authorization_fixture(
        default: &BlossomAuthorization,
        options: BlossomAuthorizationOptions,
    ) -> BlossomAuthorization {
        BlossomAuthorization {
            content: options.content.unwrap_or(default.content.clone()),
            expiration: options.expiration.unwrap_or(default.expiration),
            action: options.action.unwrap_or(default.action),
            scope: options.scope.unwrap_or(default.scope.clone()),
        }
    }

    /// Helper function to build authorization header.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    async fn build_auth_header<T>(
        signer: &T,
        authz: &BlossomAuthorization,
    ) -> Result<HeaderValue, Box<dyn Error>>
    where
        T: NostrSigner,
    {
        let pubkey = signer.get_public_key().await?;
        let auth_event = EventBuilder::blossom_auth(authz.clone())
            .build(pubkey)
            .sign(signer)
            .await?;
        let auth_bytes = serde_json::to_vec(&auth_event)?;
        let encoded_auth = base64::engine::general_purpose::STANDARD.encode(auth_bytes);
        HeaderValue::from_str(&format!("Nostr {}", encoded_auth)).map_err(From::from)
    }

    /// Helper function to extract error message from a response.
    fn extract_error(prefix: &str, response: &reqwest::Response) -> Box<dyn Error> {
        let reason = response
            .headers()
            .get("X-Reason")
            .map(|h| h.to_str().unwrap_or("Unknown reason").to_string())
            .unwrap_or_else(|| "No reason provided".to_string());
        let message = format!("{}: {} - {}", prefix, response.status(), reason);
        message.into()
    }
}

/// Options for customizing BlossomAuthorization. All fields are optional.
#[derive(Debug, Clone, Default)]
pub struct BlossomAuthorizationOptions {
    /// A human readable string explaining to the user what the events intended use is
    pub content: Option<String>,
    /// A UNIX timestamp (in seconds) indicating when the authorization should be expired
    pub expiration: Option<nostr::Timestamp>,
    /// The type of action authorized by the user
    pub action: Option<BlossomAuthorizationVerb>,
    /// The scope of the authorization
    pub scope: Option<BlossomAuthorizationScope>,
}
