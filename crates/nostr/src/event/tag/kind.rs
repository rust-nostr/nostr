// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag kind

use alloc::borrow::Cow;
use core::fmt;
use core::str::FromStr;

use crate::SingleLetterTag;

/// Tag kind
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagKind<'a> {
    /// Single letter
    SingleLetter(SingleLetterTag),
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// Human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt,
    /// Relay
    Relay,
    /// Nonce
    Nonce,
    /// Delegation
    Delegation,
    /// Content warning
    ContentWarning,
    /// Expiration
    Expiration,
    /// Subject
    Subject,
    /// Auth challenge
    Challenge,
    /// Title (NIP23)
    Title,
    /// Image (NIP23)
    Image,
    /// Thumbnail
    Thumb,
    /// Summary (NIP23)
    Summary,
    /// PublishedAt (NIP23)
    PublishedAt,
    /// Description (NIP57)
    Description,
    /// Bolt11 Invoice (NIP57)
    Bolt11,
    /// Preimage (NIP57)
    Preimage,
    /// Relays (NIP57)
    Relays,
    /// Amount (NIP57)
    Amount,
    /// Lnurl (NIP57)
    Lnurl,
    /// Name tag
    Name,
    /// Url
    Url,
    /// AES 256 GCM
    Aes256Gcm,
    /// Size of file in bytes
    Size,
    /// Size of file in pixels
    Dim,
    /// Magnet
    Magnet,
    /// Blurhash
    Blurhash,
    /// Streaming
    Streaming,
    /// Recording
    Recording,
    /// Starts
    Starts,
    /// Ends
    Ends,
    /// Status
    Status,
    /// Current participants
    CurrentParticipants,
    /// Total participants
    TotalParticipants,
    /// HTTP Method Request
    Method,
    /// Payload HASH
    Payload,
    /// Anon
    Anon,
    /// Proxy
    Proxy,
    /// Emoji
    Emoji,
    /// Encrypted
    Encrypted,
    /// Request (NIP90)
    Request,
    /// Word
    Word,
    /// Custom tag kind
    Custom(Cow<'a, str>),
}

impl<'a> TagKind<'a> {
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
            Self::SingleLetter(s) => write!(f, "{s}"),
            Self::Protected => write!(f, "-"),
            Self::Alt => write!(f, "alt"),
            Self::Relay => write!(f, "relay"),
            Self::Nonce => write!(f, "nonce"),
            Self::Delegation => write!(f, "delegation"),
            Self::ContentWarning => write!(f, "content-warning"),
            Self::Expiration => write!(f, "expiration"),
            Self::Subject => write!(f, "subject"),
            Self::Challenge => write!(f, "challenge"),
            Self::Title => write!(f, "title"),
            Self::Image => write!(f, "image"),
            Self::Thumb => write!(f, "thumb"),
            Self::Summary => write!(f, "summary"),
            Self::PublishedAt => write!(f, "published_at"),
            Self::Description => write!(f, "description"),
            Self::Bolt11 => write!(f, "bolt11"),
            Self::Preimage => write!(f, "preimage"),
            Self::Relays => write!(f, "relays"),
            Self::Amount => write!(f, "amount"),
            Self::Lnurl => write!(f, "lnurl"),
            Self::Name => write!(f, "name"),
            Self::Url => write!(f, "url"),
            Self::Aes256Gcm => write!(f, "aes-256-gcm"),
            Self::Size => write!(f, "size"),
            Self::Dim => write!(f, "dim"),
            Self::Magnet => write!(f, "magnet"),
            Self::Blurhash => write!(f, "blurhash"),
            Self::Streaming => write!(f, "streaming"),
            Self::Recording => write!(f, "recording"),
            Self::Starts => write!(f, "starts"),
            Self::Ends => write!(f, "ends"),
            Self::Status => write!(f, "status"),
            Self::CurrentParticipants => write!(f, "current_participants"),
            Self::TotalParticipants => write!(f, "total_participants"),
            Self::Method => write!(f, "method"),
            Self::Payload => write!(f, "payload"),
            Self::Anon => write!(f, "anon"),
            Self::Proxy => write!(f, "proxy"),
            Self::Emoji => write!(f, "emoji"),
            Self::Encrypted => write!(f, "encrypted"),
            Self::Request => write!(f, "request"),
            Self::Word => write!(f, "word"),
            Self::Custom(tag) => write!(f, "{tag}"),
        }
    }
}

impl<'a> From<&'a str> for TagKind<'a> {
    fn from(kind: &'a str) -> Self {
        match kind {
            "-" => Self::Protected,
            "alt" => Self::Alt,
            "relay" => Self::Relay,
            "nonce" => Self::Nonce,
            "delegation" => Self::Delegation,
            "content-warning" => Self::ContentWarning,
            "expiration" => Self::Expiration,
            "subject" => Self::Subject,
            "challenge" => Self::Challenge,
            "title" => Self::Title,
            "image" => Self::Image,
            "thumb" => Self::Thumb,
            "summary" => Self::Summary,
            "published_at" => Self::PublishedAt,
            "description" => Self::Description,
            "bolt11" => Self::Bolt11,
            "preimage" => Self::Preimage,
            "relays" => Self::Relays,
            "amount" => Self::Amount,
            "lnurl" => Self::Lnurl,
            "name" => Self::Name,
            "url" => Self::Url,
            "aes-256-gcm" => Self::Aes256Gcm,
            "size" => Self::Size,
            "dim" => Self::Dim,
            "magnet" => Self::Magnet,
            "blurhash" => Self::Blurhash,
            "streaming" => Self::Streaming,
            "recording" => Self::Recording,
            "starts" => Self::Starts,
            "ends" => Self::Ends,
            "status" => Self::Status,
            "current_participants" => Self::CurrentParticipants,
            "total_participants" => Self::TotalParticipants,
            "method" => Self::Method,
            "payload" => Self::Payload,
            "anon" => Self::Anon,
            "proxy" => Self::Proxy,
            "emoji" => Self::Emoji,
            "encrypted" => Self::Encrypted,
            "request" => Self::Request,
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
}
