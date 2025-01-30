// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP39: External Identities in Profiles
//!
//! <https://github.com/nostr-protocol/nips/blob/master/39.md>

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

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

impl FromStr for ExternalIdentity {
    type Err = Error;

    fn from_str(identity: &str) -> Result<Self, Self::Err> {
        match identity {
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
    pub fn new<S1, S2>(platform_iden: S1, proof: S2) -> Result<Self, Error>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        let i: &str = platform_iden.as_ref();
        let (platform, ident) = i.rsplit_once(':').ok_or(Error::InvalidIdentity)?;

        Ok(Self {
            platform: ExternalIdentity::from_str(platform)?,
            ident: ident.to_string(),
            proof: proof.into(),
        })
    }

    #[inline]
    pub(crate) fn tag_platform_identity(&self) -> String {
        format!("{}:{}", self.platform, self.ident)
    }
}
