// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag kind

use alloc::borrow::Cow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

use crate::{Alphabet, SingleLetterTag};

/// Tag kind
#[derive(Debug, Clone)]
pub enum TagKind<'a> {
    /// Human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt,
    /// Amount
    Amount,
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
    /// HEAD
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    Head,
    /// Branch name
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    BranchName,
    /// Merge base
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    MergeBase,
    /// Content warning
    ContentWarning,
    /// Required dependency
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Dependency,
    /// Description
    Description,
    /// Emoji
    Emoji,
    /// Encrypted
    Encrypted,
    /// Expiration
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    Expiration,
    /// File extension
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Extension,
    /// File
    File,
    /// Image
    Image,
    /// License of the shared content
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    License,
    /// Maintainers
    Maintainers,
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
    /// Preimage
    Preimage,
    /// Proxy
    Proxy,
    /// PublishedAt
    PublishedAt,
    /// Relay
    Relay,
    /// Relays
    Relays,
    /// Reference to the origin repository
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Repository,
    /// Request
    Request,
    /// Runtime or environment specification
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Runtime,
    /// Server
    Server,
    /// Status
    Status,
    /// Subject
    Subject,
    /// Summary
    Summary,
    /// Title
    Title,
    /// Thumbnail
    Thumb,
    /// Tracker
    Tracker,
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

impl PartialEq for TagKind<'_> {
    fn eq(&self, other: &TagKind<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for TagKind<'_> {}

impl PartialOrd for TagKind<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TagKind<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for TagKind<'_> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_str().hash(state);
    }
}

impl<'a> TagKind<'a> {
    /// Construct a single letter tag
    #[inline]
    pub const fn single_letter(character: Alphabet, uppercase: bool) -> Self {
        Self::SingleLetter(SingleLetterTag {
            character,
            uppercase,
        })
    }

    /// Construct `a` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::A))`.
    #[inline]
    pub const fn a() -> Self {
        Self::single_letter(Alphabet::A, false)
    }

    /// Construct `d` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D))`.
    #[inline]
    pub const fn d() -> Self {
        Self::single_letter(Alphabet::D, false)
    }

    /// Construct `e` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E))`.
    #[inline]
    pub const fn e() -> Self {
        Self::single_letter(Alphabet::E, false)
    }

    /// Construct `h` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::H))`.
    #[inline]
    pub const fn h() -> Self {
        Self::single_letter(Alphabet::H, false)
    }

    /// Construct `i` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::I))`.
    #[inline]
    pub const fn i() -> Self {
        Self::single_letter(Alphabet::I, false)
    }

    /// Construct `k` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::K))`.
    #[inline]
    pub const fn k() -> Self {
        Self::single_letter(Alphabet::K, false)
    }

    /// Construct `p` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::P))`.
    #[inline]
    pub const fn p() -> Self {
        Self::single_letter(Alphabet::P, false)
    }

    /// Construct `t` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::T))`.
    #[inline]
    pub const fn t() -> Self {
        Self::single_letter(Alphabet::T, false)
    }

    /// Construct `u` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::U))`.
    #[inline]
    pub const fn u() -> Self {
        Self::single_letter(Alphabet::U, false)
    }

    /// Construct `q` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::Q))`.
    #[inline]
    pub const fn q() -> Self {
        Self::single_letter(Alphabet::Q, false)
    }

    /// Construct `x` kind
    ///
    /// Shorthand for `TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::X))`.
    #[inline]
    pub const fn x() -> Self {
        Self::single_letter(Alphabet::X, false)
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
            Self::Alt => "alt",
            Self::Amount => "amount",
            Self::Bolt11 => "bolt11",
            Self::BranchName => "branch-name",
            Self::Challenge => "challenge",
            Self::Client => "client",
            Self::Clone => "clone",
            Self::Commit => "commit",
            Self::ContentWarning => "content-warning",
            Self::Dependency => "dep",
            Self::Description => "description",
            Self::Emoji => "emoji",
            Self::Encrypted => "encrypted",
            Self::Expiration => "expiration",
            Self::Extension => "extension",
            Self::File => "file",
            Self::Head => "HEAD",
            Self::Image => "image",
            Self::License => "license",
            Self::Maintainers => "maintainers",
            Self::MergeBase => "merge-base",
            Self::MlsProtocolVersion => "mls_protocol_version",
            Self::MlsCiphersuite => "mls_ciphersuite",
            Self::MlsExtensions => "mls_extensions",
            Self::Name => "name",
            Self::Nonce => "nonce",
            Self::Preimage => "preimage",
            Self::Proxy => "proxy",
            Self::PublishedAt => "published_at",
            Self::Relay => "relay",
            Self::Relays => "relays",
            Self::Repository => "repo",
            Self::Request => "request",
            Self::Runtime => "runtime",
            Self::Server => "server",
            Self::Status => "status",
            Self::Subject => "subject",
            Self::Summary => "summary",
            Self::Title => "title",
            Self::Thumb => "thumb",
            Self::Tracker => "tracker",
            Self::Url => "url",
            Self::Web => "web",
            Self::Word => "word",
            Self::SingleLetter(s) => s.as_str(),
            Self::Custom(tag) => tag.as_ref(),
        }
    }
}

impl fmt::Display for TagKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> From<&'a str> for TagKind<'a> {
    fn from(kind: &'a str) -> Self {
        match kind {
            "alt" => Self::Alt,
            "amount" => Self::Amount,
            "bolt11" => Self::Bolt11,
            "branch-name" => Self::BranchName,
            "challenge" => Self::Challenge,
            "client" => Self::Client,
            "clone" => Self::Clone,
            "commit" => Self::Commit,
            "content-warning" => Self::ContentWarning,
            "dep" => Self::Dependency,
            "description" => Self::Description,
            "emoji" => Self::Emoji,
            "encrypted" => Self::Encrypted,
            "expiration" => Self::Expiration,
            "extension" => Self::Extension,
            "file" => Self::File,
            "image" => Self::Image,
            "license" => Self::License,
            "maintainers" => Self::Maintainers,
            "merge-base" => Self::MergeBase,
            "mls_protocol_version" => Self::MlsProtocolVersion,
            "mls_ciphersuite" => Self::MlsCiphersuite,
            "mls_extensions" => Self::MlsExtensions,
            "name" => Self::Name,
            "nonce" => Self::Nonce,
            "preimage" => Self::Preimage,
            "proxy" => Self::Proxy,
            "published_at" => Self::PublishedAt,
            "relay" => Self::Relay,
            "relays" => Self::Relays,
            "repo" => Self::Repository,
            "request" => Self::Request,
            "runtime" => Self::Runtime,
            "HEAD" => Self::Head,
            "server" => Self::Server,
            "status" => Self::Status,
            "subject" => Self::Subject,
            "summary" => Self::Summary,
            "title" => Self::Title,
            "thumb" => Self::Thumb,
            "tracker" => Self::Tracker,
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
    use alloc::string::String;

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
    fn test_de_serialization() {
        assert_eq!(TagKind::from("alt"), TagKind::Alt);
        assert_eq!(TagKind::Alt.as_str(), "alt");

        assert_eq!(TagKind::from("amount"), TagKind::Amount);
        assert_eq!(TagKind::Amount.as_str(), "amount");

        assert_eq!(TagKind::from("clone"), TagKind::Clone);
        assert_eq!(TagKind::Clone.as_str(), "clone");

        assert_eq!(TagKind::from("dep"), TagKind::Dependency);
        assert_eq!(TagKind::Dependency.as_str(), "dep");

        assert_eq!(TagKind::from("expiration"), TagKind::Expiration);
        assert_eq!(TagKind::Expiration.as_str(), "expiration");

        assert_eq!(TagKind::from("extension"), TagKind::Extension);
        assert_eq!(TagKind::Extension.as_str(), "extension");

        assert_eq!(TagKind::from("file"), TagKind::File);
        assert_eq!(TagKind::File.as_str(), "file");

        assert_eq!(TagKind::from("HEAD"), TagKind::Head);
        assert_eq!(TagKind::Head.as_str(), "HEAD");

        assert_eq!(TagKind::from("license"), TagKind::License);
        assert_eq!(TagKind::License.as_str(), "license");

        assert_eq!(TagKind::from("maintainers"), TagKind::Maintainers);
        assert_eq!(TagKind::Maintainers.as_str(), "maintainers");

        assert_eq!(TagKind::from("repo"), TagKind::Repository);
        assert_eq!(TagKind::Repository.as_str(), "repo");

        assert_eq!(TagKind::from("runtime"), TagKind::Runtime);
        assert_eq!(TagKind::Runtime.as_str(), "runtime");

        assert_eq!(TagKind::from("tracker"), TagKind::Tracker);
        assert_eq!(TagKind::Tracker.as_str(), "tracker");

        assert_eq!(TagKind::from("web"), TagKind::Web);
        assert_eq!(TagKind::Web.as_str(), "web");

        assert_eq!(TagKind::from("a"), TagKind::a());
        assert_eq!(TagKind::a().as_str(), "a");

        assert_eq!(TagKind::from("e"), TagKind::e());
        assert_eq!(TagKind::e().as_str(), "e");

        assert_eq!(TagKind::from("p"), TagKind::p());
        assert_eq!(TagKind::p().as_str(), "p");
    }

    #[test]
    fn test_eq() {
        assert_eq!(TagKind::Custom(Cow::from("p")), TagKind::p());
        assert_eq!(
            TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::P)),
            TagKind::p()
        );
        assert_eq!(TagKind::Custom(Cow::from("e")), TagKind::e());
        assert_eq!(
            TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E)),
            TagKind::e()
        );
    }
}
