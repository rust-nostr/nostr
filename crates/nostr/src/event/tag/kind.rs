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
    /// HTTP Method Request
    Method,
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
}

impl<'a> fmt::Display for TagKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Aes256Gcm => write!(f, "aes-256-gcm"),
            Self::Alt => write!(f, "alt"),
            Self::Amount => write!(f, "amount"),
            Self::Anon => write!(f, "anon"),
            Self::Blurhash => write!(f, "blurhash"),
            Self::Bolt11 => write!(f, "bolt11"),
            Self::Challenge => write!(f, "challenge"),
            Self::Client => write!(f, "client"),
            Self::ContentWarning => write!(f, "content-warning"),
            Self::CurrentParticipants => write!(f, "current_participants"),
            Self::Delegation => write!(f, "delegation"),
            Self::Description => write!(f, "description"),
            Self::Dim => write!(f, "dim"),
            Self::Emoji => write!(f, "emoji"),
            Self::Encrypted => write!(f, "encrypted"),
            Self::Ends => write!(f, "ends"),
            Self::Expiration => write!(f, "expiration"),
            Self::Image => write!(f, "image"),
            Self::Lnurl => write!(f, "lnurl"),
            Self::Magnet => write!(f, "magnet"),
            Self::Method => write!(f, "method"),
            Self::Name => write!(f, "name"),
            Self::Nonce => write!(f, "nonce"),
            Self::Payload => write!(f, "payload"),
            Self::Preimage => write!(f, "preimage"),
            Self::Protected => write!(f, "-"),
            Self::Proxy => write!(f, "proxy"),
            Self::PublishedAt => write!(f, "published_at"),
            Self::Recording => write!(f, "recording"),
            Self::Relay => write!(f, "relay"),
            Self::Relays => write!(f, "relays"),
            Self::Request => write!(f, "request"),
            Self::Size => write!(f, "size"),
            Self::Starts => write!(f, "starts"),
            Self::Status => write!(f, "status"),
            Self::Streaming => write!(f, "streaming"),
            Self::Subject => write!(f, "subject"),
            Self::Summary => write!(f, "summary"),
            Self::Title => write!(f, "title"),
            Self::Thumb => write!(f, "thumb"),
            Self::TotalParticipants => write!(f, "total_participants"),
            Self::Url => write!(f, "url"),
            Self::Web => write!(f, "web"),
            Self::Word => write!(f, "word"),
            Self::SingleLetter(s) => write!(f, "{s}"),
            Self::Custom(tag) => write!(f, "{tag}"),
        }
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
            "method" => Self::Method,
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
        assert_eq!(TagKind::from("web"), TagKind::Web);
        assert_eq!(TagKind::Web.to_string(), "web");
    }
}
