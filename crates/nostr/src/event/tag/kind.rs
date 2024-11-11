// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag kind

use alloc::borrow::Cow;
use core::fmt;
use core::str::FromStr;

use crate::{Alphabet, SingleLetterTag};

/// Tag kind
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagKind<'a> {
    /// AES 256 GCM
    Aes256Gcm,
    /// Human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt,
    /// Amount
    Amount,
    /// Anonymous
    Anon,
    /// Blurhash
    Blurhash,
    /// Bolt11 invoice
    Bolt11,
    /// Challenge
    Challenge,
    /// Client
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/89.md>
    Client,
    /// Clone
    Clone,
    /// Commit
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    Commit,
    /// Content warning
    ContentWarning,
    /// Current participants
    CurrentParticipants,
    /// Delegation
    Delegation,
    /// Description
    Description,
    /// Size of file in pixels
    Dim,
    /// Emoji
    Emoji,
    /// Encrypted
    Encrypted,
    /// Ends
    Ends,
    /// Expiration
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    Expiration,
    /// Image
    Image,
    /// Lnurl
    Lnurl,
    /// Magnet
    Magnet,
    /// Maintainers
    Maintainers,
    /// HTTP Method Request
    Method,
    /// MLS Protocol Version
    MlsProtocolVersion,
    /// MLS Cipher Suite
    MlsCiphersuite,
    /// MLS Extensions
    MlsExtensions,
    /// Name
    Name,
    /// Nonce
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    Nonce,
    /// Payload
    Payload,
    /// Preimage
    Preimage,
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// Proxy
    Proxy,
    /// PublishedAt
    PublishedAt,
    /// Recording
    Recording,
    /// Relay
    Relay,
    /// Relays
    Relays,
    /// Request
    Request,
    /// Size of file in bytes
    Size,
    /// Starts
    Starts,
    /// Status
    Status,
    /// Streaming
    Streaming,
    /// Subject
    Subject,
    /// Summary
    Summary,
    /// Title
    Title,
    /// Thumbnail
    Thumb,
    /// Total participants
    TotalParticipants,
    /// Url
    Url,
    /// Web
    Web,
    /// Word
    Word,
    /// Single letter
    SingleLetter(SingleLetterTag),
    /// Custom
    Custom(Cow<'a, str>),
}

impl<'a> TagKind<'a> {
    /// Construct `a` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::A))`.
    #[inline]
    pub fn a() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::A))
    }

    /// Construct `d` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D))`.
    #[inline]
    pub fn d() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::D))
    }

    /// Construct `e` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E))`.
    #[inline]
    pub fn e() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::E))
    }

    /// Construct `h` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::H))`.
    #[inline]
    pub fn h() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::H))
    }

    /// Construct `p` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::P))`.
    #[inline]
    pub fn p() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::P))
    }

    /// Construct `t` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::T))`.
    #[inline]
    pub fn t() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::T))
    }

    /// Construct `q` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::Q))`.
    #[inline]
    pub fn q() -> Self {
        Self::SingleLetter(SingleLetterTag::lowercase(Alphabet::Q))
    }

    /// Construct [`TagKind::Custom`]
    ///
    /// Shorthand for `TagKind::Custom(Cow::from(...))`.
    #[inline]
    pub fn custom<T>(kind: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self::Custom(kind.into())
    }

    /// Convert to `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Aes256Gcm => "aes-256-gcm",
            Self::Alt => "alt",
            Self::Amount => "amount",
            Self::Anon => "anon",
            Self::Blurhash => "blurhash",
            Self::Bolt11 => "bolt11",
            Self::Challenge => "challenge",
            Self::Client => "client",
            Self::Clone => "clone",
            Self::Commit => "commit",
            Self::ContentWarning => "content-warning",
            Self::CurrentParticipants => "current_participants",
            Self::Delegation => "delegation",
            Self::Description => "description",
            Self::Dim => "dim",
            Self::Emoji => "emoji",
            Self::Encrypted => "encrypted",
            Self::Ends => "ends",
            Self::Expiration => "expiration",
            Self::Image => "image",
            Self::Lnurl => "lnurl",
            Self::Magnet => "magnet",
            Self::Maintainers => "maintainers",
            Self::Method => "method",
            Self::MlsProtocolVersion => "mls_protocol_version",
            Self::MlsCiphersuite => "mls_ciphersuite",
            Self::MlsExtensions => "mls_extensions",
            Self::Name => "name",
            Self::Nonce => "nonce",
            Self::Payload => "payload",
            Self::Preimage => "preimage",
            Self::Protected => "-",
            Self::Proxy => "proxy",
            Self::PublishedAt => "published_at",
            Self::Recording => "recording",
            Self::Relay => "relay",
            Self::Relays => "relays",
            Self::Request => "request",
            Self::Size => "size",
            Self::Starts => "starts",
            Self::Status => "status",
            Self::Streaming => "streaming",
            Self::Subject => "subject",
            Self::Summary => "summary",
            Self::Title => "title",
            Self::Thumb => "thumb",
            Self::TotalParticipants => "total_participants",
            Self::Url => "url",
            Self::Web => "web",
            Self::Word => "word",
            Self::SingleLetter(s) => s.as_str(),
            Self::Custom(tag) => tag.as_ref(),
        }
    }
}

impl<'a> fmt::Display for TagKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> From<&'a str> for TagKind<'a> {
    fn from(kind: &'a str) -> Self {
        match kind {
            "-" => Self::Protected,
            "aes-256-gcm" => Self::Aes256Gcm,
            "alt" => Self::Alt,
            "amount" => Self::Amount,
            "anon" => Self::Anon,
            "blurhash" => Self::Blurhash,
            "bolt11" => Self::Bolt11,
            "challenge" => Self::Challenge,
            "client" => Self::Client,
            "clone" => Self::Clone,
            "commit" => Self::Commit,
            "content-warning" => Self::ContentWarning,
            "current_participants" => Self::CurrentParticipants,
            "delegation" => Self::Delegation,
            "description" => Self::Description,
            "dim" => Self::Dim,
            "emoji" => Self::Emoji,
            "encrypted" => Self::Encrypted,
            "ends" => Self::Ends,
            "expiration" => Self::Expiration,
            "image" => Self::Image,
            "lnurl" => Self::Lnurl,
            "magnet" => Self::Magnet,
            "maintainers" => Self::Maintainers,
            "method" => Self::Method,
            "mls_protocol_version" => Self::MlsProtocolVersion,
            "mls_ciphersuite" => Self::MlsCiphersuite,
            "mls_extensions" => Self::MlsExtensions,
            "name" => Self::Name,
            "nonce" => Self::Nonce,
            "payload" => Self::Payload,
            "preimage" => Self::Preimage,
            "proxy" => Self::Proxy,
            "published_at" => Self::PublishedAt,
            "recording" => Self::Recording,
            "relay" => Self::Relay,
            "relays" => Self::Relays,
            "request" => Self::Request,
            "size" => Self::Size,
            "starts" => Self::Starts,
            "status" => Self::Status,
            "streaming" => Self::Streaming,
            "subject" => Self::Subject,
            "summary" => Self::Summary,
            "title" => Self::Title,
            "thumb" => Self::Thumb,
            "total_participants" => Self::TotalParticipants,
            "url" => Self::Url,
            "web" => Self::Web,
            "word" => Self::Word,
            k => match SingleLetterTag::from_str(k) {
                Ok(s) => Self::SingleLetter(s),
                Err(..) => Self::Custom(Cow::Borrowed(k)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString};

    use super::*;

    #[test]
    fn test_custom_tag_kind_constructor() {
        let owned = TagKind::custom(String::from("owned"));
        match owned {
            TagKind::Custom(Cow::Owned(val)) => assert_eq!(val, "owned"),
            _ => panic!("Unexpected tag kind"),
        };

        let borrowed = TagKind::custom("borrowed");
        match borrowed {
            TagKind::Custom(Cow::Borrowed(val)) => assert_eq!(val, "borrowed"),
            _ => panic!("Unexpected tag kind"),
        };
    }

    #[test]
    fn test_from_to_tag_kind() {
        assert_eq!(TagKind::from("clone"), TagKind::Clone);
        assert_eq!(TagKind::Clone.to_string(), "clone");

        assert_eq!(TagKind::from("maintainers"), TagKind::Maintainers);
        assert_eq!(TagKind::Maintainers.to_string(), "maintainers");

        assert_eq!(TagKind::from("web"), TagKind::Web);
        assert_eq!(TagKind::Web.to_string(), "web");
    }
}
