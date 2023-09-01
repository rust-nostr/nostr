// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP48
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
        match self {
            Self::ActivityPub => write!(f, "activitypub"),
            Self::ATProto => write!(f, "atproto"),
            Self::Rss => write!(f, "rss"),
            Self::Web => write!(f, "web"),
            Self::Custom(m) => write!(f, "{m}"),
        }
    }
}

impl<S> From<S> for Protocol
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        let s: String = s.into();
        match s.as_str() {
            "activitypub" => Self::ActivityPub,
            "atproto" => Self::ATProto,
            "rss" => Self::Rss,
            "web" => Self::Web,
            s => Self::Custom(s.to_string()),
        }
    }
}
