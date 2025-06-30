//! Implements a Blossom client for interacting with Blossom servers

use std::time::Duration;

use base64::engine::general_purpose;
use base64::Engine;
use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::hashes::Hash;
use nostr::signer::NostrSigner;
use nostr::{Event, EventBuilder, JsonUtil, PublicKey, Timestamp, Url};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, RANGE};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::redirect::Policy;
use reqwest::{Response, StatusCode};

use crate::bud01::{
    BlossomAuthorization, BlossomAuthorizationScope, BlossomAuthorizationVerb,
    BlossomBuilderExtension,
};
use crate::bud02::BlobDescriptor;
use crate::error::Error;

/// A client for interacting with a Blossom server
///
/// <https://github.com/hzrd149/blossom>
#[derive(Debug, Clone)]
pub struct BlossomClient {
    base_url: Url,
    client: reqwest::Client,
}

impl BlossomClient {
    /// Creates a new `BlossomClient` with the given base URL.
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
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
    ) -> Result<BlobDescriptor, Error>
    where
        T: NostrSigner,
    {
        let url = format!("{}upload", self.base_url);

        let hash: Sha256Hash = Sha256Hash::hash(&data);
        let file_hashes: Vec<Sha256Hash> = vec![hash];

        let mut request = self.client.put(url).body(data);
        let mut headers = HeaderMap::new();

        if let Some(ct) = content_type {
            headers.insert(CONTENT_TYPE, HeaderValue::from_str(&ct)?);
        }

        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::Upload,
                "Blossom upload authorization",
                BlossomAuthorizationScope::BlobSha256Hashes(file_hashes),
            );
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
        }

        request = request.headers(headers);

        let response: Response = request.send().await?;

        match response.status() {
            StatusCode::OK => {
                let descriptor: BlobDescriptor = response.json().await?;
                Ok(descriptor)
            }
            _ => Err(Error::response("Failed to upload blob", response)),
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
    ) -> Result<Vec<BlobDescriptor>, Error>
    where
        T: NostrSigner,
    {
        let mut url = format!("{}list/{}", self.base_url, pubkey.to_hex());

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

        let mut request = self.client.get(url);
        let mut headers = HeaderMap::new();

        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::List,
                "Blossom list authorization",
                BlossomAuthorizationScope::ServerUrl(self.base_url.clone()),
            );
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
        }

        request = request.headers(headers);

        let response: Response = request.send().await?;

        match response.status() {
            StatusCode::OK => {
                let descriptors: Vec<BlobDescriptor> = response.json().await?;
                Ok(descriptors)
            }
            _ => Err(Error::response("Failed to list blobs", response)),
        }
    }

    /// Retrieves a blob from the Blossom server, with optional authorization.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    pub async fn get_blob<T>(
        &self,
        sha256: Sha256Hash,
        range: Option<String>,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: Option<&T>,
    ) -> Result<Vec<u8>, Error>
    where
        T: NostrSigner,
    {
        let url = format!("{}{}", self.base_url, sha256);
        let mut request = self.client.get(url);
        let mut headers = HeaderMap::new();

        if let Some(range_value) = range {
            headers.insert(RANGE, HeaderValue::from_str(&range_value)?);
        }

        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::Get,
                "Blossom get authorization",
                BlossomAuthorizationScope::BlobSha256Hashes(vec![sha256]),
            );
            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);
        }

        request = request.headers(headers);

        let response: Response = request.send().await?;

        if response.status().is_redirection() {
            match response.headers().get("Location") {
                Some(location) => {
                    let location_str: &str = location.to_str()?;
                    if !location_str.contains(&sha256.to_string()) {
                        return Err(Error::RedirectUrlDoesNotContainSha256);
                    }
                }
                None => return Err(Error::RedirectResponseMissingLocationHeader),
            }
        }

        match response.status() {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => Ok(response.bytes().await?.to_vec()),
            _ => Err(Error::response("Failed to get blob", response)),
        }
    }

    /// Checks if a blob exists on the Blossom server.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    pub async fn has_blob<T>(
        &self,
        sha256: Sha256Hash,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: Option<&T>,
    ) -> Result<bool, Error>
    where
        T: NostrSigner,
    {
        let url = format!("{}{}", self.base_url, sha256);

        let mut request = self.client.head(url);

        if let Some(signer) = signer {
            let default_auth = self.default_auth(
                BlossomAuthorizationVerb::Get,
                "Blossom get authorization",
                BlossomAuthorizationScope::BlobSha256Hashes(vec![sha256]),
            );

            let final_auth = authorization_options
                .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
                .unwrap_or(default_auth);

            let mut headers = HeaderMap::new();
            let auth_header = Self::build_auth_header(signer, &final_auth).await?;
            headers.insert(AUTHORIZATION, auth_header);

            request = request.headers(headers);
        }

        let response: Response = request.send().await?;

        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Err(Error::response("Unexpected HTTP status code", response)),
        }
    }

    /// Deletes a blob from the Blossom server.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/02.md>
    pub async fn delete_blob<T>(
        &self,
        sha256: Sha256Hash,
        authorization_options: Option<BlossomAuthorizationOptions>,
        signer: &T,
    ) -> Result<(), Error>
    where
        T: NostrSigner,
    {
        let url = format!("{}{}", self.base_url, sha256);

        let mut headers = HeaderMap::new();
        let default_auth = self.default_auth(
            BlossomAuthorizationVerb::Delete,
            "Blossom delete authorization",
            BlossomAuthorizationScope::BlobSha256Hashes(vec![sha256]),
        );

        let final_auth = authorization_options
            .map(|opts| Self::update_authorization_fixture(&default_auth, opts))
            .unwrap_or(default_auth);

        let auth_header = Self::build_auth_header(signer, &final_auth).await?;
        headers.insert(AUTHORIZATION, auth_header);

        let response: Response = self.client.delete(url).headers(headers).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::response("Failed to delete blob", response))
        }
    }

    /// Returns a default BlossomAuthorization object based on the parameters provided.
    fn default_auth<T>(
        &self,
        action: BlossomAuthorizationVerb,
        default_content: T,
        default_scope: BlossomAuthorizationScope,
    ) -> BlossomAuthorization
    where
        T: Into<String>,
    {
        let expiration_timestamp: Timestamp = Timestamp::now() + Duration::from_secs(300);
        BlossomAuthorization::new(
            default_content.into(),
            expiration_timestamp,
            action,
            default_scope,
        )
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
    ) -> Result<HeaderValue, Error>
    where
        T: NostrSigner,
    {
        let auth_event: Event = EventBuilder::blossom_auth(authz.clone())
            .sign(signer)
            .await?;
        let encoded_auth: String = general_purpose::STANDARD.encode(auth_event.as_json());
        let value: String = format!("Nostr {}", encoded_auth);
        Ok(HeaderValue::try_from(value)?)
    }
}

/// Options for customizing BlossomAuthorization. All fields are optional.
#[derive(Debug, Clone, Default)]
pub struct BlossomAuthorizationOptions {
    /// A human readable string explaining to the user what the events intended use is
    pub content: Option<String>,
    /// A UNIX timestamp (in seconds) indicating when the authorization should be expired
    pub expiration: Option<Timestamp>,
    /// The type of action authorized by the user
    pub action: Option<BlossomAuthorizationVerb>,
    /// The scope of the authorization
    pub scope: Option<BlossomAuthorizationScope>,
}
