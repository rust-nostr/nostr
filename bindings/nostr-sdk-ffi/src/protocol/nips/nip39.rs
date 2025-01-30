// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip39;
use uniffi::{Enum, Record};

/// Supported external identity providers
///
/// <https://github.com/nostr-protocol/nips/blob/master/39.md>
#[derive(Enum)]
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

impl From<ExternalIdentity> for nip39::ExternalIdentity {
    fn from(value: ExternalIdentity) -> Self {
        match value {
            ExternalIdentity::GitHub => Self::GitHub,
            ExternalIdentity::Twitter => Self::Twitter,
            ExternalIdentity::Mastodon => Self::Mastodon,
            ExternalIdentity::Telegram => Self::Telegram,
        }
    }
}

impl From<nip39::ExternalIdentity> for ExternalIdentity {
    fn from(value: nip39::ExternalIdentity) -> Self {
        match value {
            nip39::ExternalIdentity::GitHub => Self::GitHub,
            nip39::ExternalIdentity::Twitter => Self::Twitter,
            nip39::ExternalIdentity::Mastodon => Self::Mastodon,
            nip39::ExternalIdentity::Telegram => Self::Telegram,
        }
    }
}

/// External identity
///
/// <https://github.com/nostr-protocol/nips/blob/master/39.md>
#[derive(Record)]
pub struct Identity {
    /// The external identity provider
    pub platform: ExternalIdentity,
    /// The user's identity (username) on the provider
    pub ident: String,
    /// The user's proof on the provider
    pub proof: String,
}

impl From<Identity> for nip39::Identity {
    fn from(value: Identity) -> Self {
        Self {
            platform: value.platform.into(),
            ident: value.ident,
            proof: value.proof,
        }
    }
}

impl From<nip39::Identity> for Identity {
    fn from(value: nip39::Identity) -> Self {
        Self {
            platform: value.platform.into(),
            ident: value.ident,
            proof: value.proof,
        }
    }
}
