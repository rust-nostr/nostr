// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP48: Proxy Tags
//!
//! <https://github.com/nostr-protocol/nips/blob/master/48.md>

use alloc::string::{String, ToString};
use core::fmt;

/// NIP48 Proxy Protocol
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Protocol {
    /// ActivityPub
    ActivityPub,
    /// AT Protocol
    ATProto,
    /// Rss
    Rss,
    /// Web
    Web,
    /// Custom
    Custom(String),
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Protocol {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::ActivityPub => "activitypub",
            Self::ATProto => "atproto",
            Self::Rss => "rss",
            Self::Web => "web",
            Self::Custom(m) => m.as_str(),
        }
    }
}

impl<S> From<S> for Protocol
where
    S: AsRef<str>,
{
    fn from(s: S) -> Self {
        match s.as_ref() {
            "activitypub" => Self::ActivityPub,
            "atproto" => Self::ATProto,
            "rss" => Self::Rss,
            "web" => Self::Web,
            s => Self::Custom(s.to_string()),
        }
    }
}
