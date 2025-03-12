use std::fmt;

use nostr::hashes::sha256::Hash as Sha256Hash;

/// Represents the authorization data for accessing a Blossom server.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlossomAuthorization {
    /// A human readable string explaining to the user what the events intended use is
    pub content: String,
    /// A UNIX timestamp (in seconds) indicating when the authorization should be expired
    pub expiration: nostr::Timestamp,
    /// The type of action authorized by the user
    pub action: BlossomAuthorizationVerb,
    /// The scope of the authorization
    pub scope: BlossomAuthorizationScope,
}

/// The scope of a Blossom authorization event
///
/// MUST contain either a server tag containing the full URL to the server or MUST contain at least one x tag matching the sha256 hash of the blob being retrieved
///
/// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlossomAuthorizationScope {
    /// Authorizes access to blobs with the given SHA256 hashes.
    BlobSha256Hashes(Vec<Sha256Hash>),
    /// Authorizes access to the given server URL.
    ServerUrl(String),
}

/// Represents the possible actions that can be authorized by a Blossom authorization event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlossomAuthorizationVerb {
    /// Authorizes the retrieval of a blob.
    Get,
    /// Authorizes the upload of a blob.
    Upload,
    /// Authorizes the listing of blobs.
    List,
    /// Authorizes the deletion of a blob.
    Delete,
}

impl fmt::Display for BlossomAuthorizationVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl BlossomAuthorizationVerb {
    /// Converts the authorization verb into a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get => "get",
            Self::Upload => "upload",
            Self::List => "list",
            Self::Delete => "delete",
        }
    }
}

/// An extension trait for `nostr::EventBuilder` to add Blossom authorization functionality.
pub trait BlossomBuilderExtension {
    /// Creates a Blossom authorization event.
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    fn blossom_auth(authorization: BlossomAuthorization) -> Self;
}

impl Into<Vec<nostr::Tag>> for BlossomAuthorization {
    fn into(self) -> Vec<nostr::Tag> {
        let mut tags = Vec::new();
        tags.extend(Into::<Vec<nostr::Tag>>::into(self.scope));
        tags.push(nostr::Tag::from_standardized(
            nostr::event::TagStandard::Expiration(self.expiration),
        ));
        // Add the 't' tag to say what this auth is for
        tags.push(nostr::Tag::from_standardized(
            nostr::event::TagStandard::Hashtag(self.action.to_string()),
        ));
        tags
    }
}

impl Into<Vec<nostr::Tag>> for BlossomAuthorizationScope {
    fn into(self) -> Vec<nostr::Tag> {
        let mut tags = Vec::new();
        match self {
            BlossomAuthorizationScope::BlobSha256Hashes(hashes) => {
                for hash in hashes {
                    tags.push(nostr::Tag::from_standardized(
                        nostr::event::TagStandard::Sha256(hash),
                    ));
                }
            }
            BlossomAuthorizationScope::ServerUrl(url) => {
                tags.push(nostr::event::Tag::from_standardized(
                    nostr::TagStandard::Server(url),
                ));
            }
        }
        tags
    }
}

impl BlossomBuilderExtension for nostr::EventBuilder {
    /// Blossom authorization event
    ///
    /// <https://github.com/hzrd149/blossom/blob/master/buds/01.md>
    #[inline]
    fn blossom_auth(authorization: BlossomAuthorization) -> Self {
        let tags: Vec<nostr::Tag> = authorization.clone().into();
        Self::new(nostr::Kind::BlossomAuth, authorization.content).tags(tags)
    }
}

impl BlossomAuthorization {
    /// Constructor for creating a new BlossomAuthorization
    pub fn new(
        content: String,
        expiration: nostr::Timestamp,
        action: BlossomAuthorizationVerb,
        scope: BlossomAuthorizationScope,
    ) -> Self {
        Self {
            content,
            expiration,
            action,
            scope,
        }
    }
}
