// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-10
//!
//! <https://github.com/nostr-protocol/nips/blob/master/10.md>

use alloc::string::{String, ToString};
use core::fmt;

/// Marker
///
/// <https://github.com/nostr-protocol/nips/blob/master/10.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Mention
    Mention,
    /// Custom
    Custom(String),
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => write!(f, "root"),
            Self::Reply => write!(f, "reply"),
            Self::Mention => write!(f, "mention"),
            Self::Custom(m) => write!(f, "{m}"),
        }
    }
}

impl<S> From<S> for Marker
where
    S: AsRef<str>,
{
    fn from(s: S) -> Self {
        match s.as_ref() {
            "root" => Self::Root,
            "reply" => Self::Reply,
            "mention" => Self::Mention,
            v => Self::Custom(v.to_string()),
        }
    }
}
