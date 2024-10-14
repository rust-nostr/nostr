// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP39
//!
//! <https://github.com/nostr-protocol/nips/blob/master/39.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::{Alphabet, SingleLetterTag, TagKind, TagStandard};

/// NIP56 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid identity
    InvalidIdentity,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentity => write!(f, "Invalid identity tag"),
        }
    }
}

/// Supported external identity providers
///
/// <https://github.com/nostr-protocol/nips/blob/master/39.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalIdentity {
    /// github.com
    GitHub,
    /// twitter.com
    Twitter,
    /// mastodon.social
    Mastodon,
    /// telegram.org
    Telegram,
}

impl fmt::Display for ExternalIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::Twitter => write!(f, "twitter"),
            Self::Mastodon => write!(f, "mastodon"),
            Self::Telegram => write!(f, "telegram"),
        }
    }
}

impl TryFrom<String> for ExternalIdentity {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "github" => Ok(Self::GitHub),
            "twitter" => Ok(Self::Twitter),
            "mastodon" => Ok(Self::Mastodon),
            "telegram" => Ok(Self::Telegram),
            _ => Err(Error::InvalidIdentity),
        }
    }
}

/// External identity
///
/// <https://github.com/nostr-protocol/nips/blob/master/39.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    /// The external identity provider
    pub platform: ExternalIdentity,
    /// The user's identity (username) on the provider
    pub ident: String,
    /// The user's proof on the provider
    pub proof: String,
}

impl Identity {
    /// Construct new identity
    pub fn new<S>(platform_iden: S, proof: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let i: String = platform_iden.into();
        let (platform, ident) = i.rsplit_once(':').ok_or(Error::InvalidIdentity)?;
        let platform: ExternalIdentity = platform.to_string().try_into()?;

        Ok(Self {
            platform,
            ident: ident.to_string(),
            proof: proof.into(),
        })
    }
}

impl TryFrom<TagStandard> for Identity {
    type Error = Error;

    fn try_from(value: TagStandard) -> Result<Self, Self::Error> {
        match value {
            TagStandard::ExternalIdentity(iden) => Ok(iden),
            _ => Err(Error::InvalidIdentity),
        }
    }
}

impl From<Identity> for TagStandard {
    fn from(value: Identity) -> Self {
        Self::ExternalIdentity(value)
    }
}

impl From<Identity> for Vec<String> {
    fn from(value: Identity) -> Self {
        vec![
            TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::I,
                uppercase: false,
            })
            .to_string(),
            format!("{}:{}", value.platform, value.ident),
            value.proof,
        ]
    }
}
