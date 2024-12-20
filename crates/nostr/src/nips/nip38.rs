//! NIP-38: User Statuses
//!
//! This NIP enables a way for users to share live statuses such as what music
//! they are listening to, as well as what they are currently doing.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/38.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::alloc::string::ToString;
use crate::{EventBuilder, EventId, Kind, PublicKey, Tag, TagKind, Timestamp, Url};

/// User Status Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusType {
    /// General status (working, away, etc)
    General,
    /// Music status (currently playing)
    Music,
    /// Custom status type
    Custom(String),
}

impl From<&str> for StatusType {
    fn from(s: &str) -> Self {
        match s {
            "general" => Self::General,
            "music" => Self::Music,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for StatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General => write!(f, "general"),
            Self::Music => write!(f, "music"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// User Status
#[derive(Debug, Clone)]
pub struct UserStatus {
    /// Status content
    pub content: String,
    /// Status type
    pub status_type: StatusType,
    /// Optional URL reference
    pub url: Option<Url>,
    /// Optional profile reference
    pub profile: Option<PublicKey>,
    /// Optional note reference
    pub note: Option<EventId>,
    /// Optional expiration
    pub expiration: Option<Timestamp>,
}

impl UserStatus {
    /// Create new user status
    pub fn new<S: Into<String>>(content: S, status_type: StatusType) -> Self {
        Self {
            content: content.into(),
            status_type,
            url: None,
            profile: None,
            note: None,
            expiration: None,
        }
    }

    /// Convert the user status into an event builder
    pub fn to_event_builder(self) -> EventBuilder {
        let mut tags = Vec::new();

        // Add status type
        tags.push(Tag::custom(
            TagKind::Custom("d".into()),
            vec![self.status_type.to_string()],
        ));

        // Add URL if present
        if let Some(url) = self.url {
            tags.push(Tag::reference(url));
        }

        // Add profile if present
        if let Some(pubkey) = self.profile {
            tags.push(Tag::public_key(pubkey));
        }

        // Add note if present
        if let Some(event_id) = self.note {
            tags.push(Tag::event(event_id));
        }

        // Add expiration if present
        if let Some(expiration) = self.expiration {
            tags.push(Tag::expiration(expiration));
        }

        EventBuilder::new(Kind::UserStatus(0), self.content).tags(tags)
    }
}
