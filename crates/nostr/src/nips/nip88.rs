// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP88: Polls
//!
//! <https://github.com/nostr-protocol/nips/blob/master/88.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::{EventBuilder, Kind, RelayUrl, Tag, TagStandard, Timestamp};

/// NIP88 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Unknown poll type
    UnknownPollType,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPollType => write!(f, "unknown poll type"),
        }
    }
}

/// Poll type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PollType {
    /// Single choice
    SingleChoice,
    /// Multiple choice
    MultipleChoice,
}

impl fmt::Display for PollType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PollType {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::SingleChoice => "singlechoice",
            Self::MultipleChoice => "multiplechoice",
        }
    }
}

impl FromStr for PollType {
    type Err = Error;

    fn from_str(poll_type: &str) -> Result<Self, Self::Err> {
        match poll_type {
            "singlechoice" => Ok(Self::SingleChoice),
            "multiplechoice" => Ok(Self::MultipleChoice),
            _ => Err(Error::UnknownPollType),
        }
    }
}

/// Poll option
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PollOption {
    /// Option ID
    id: String,
    /// Option label
    text: String,
}

/// Poll
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Poll {
    /// Poll title
    title: String,
    /// Poll type
    r#type: PollType,
    /// Poll options
    options: Vec<PollOption>,
    /// Relays
    relays: Vec<RelayUrl>,
    /// Optionally, when the poll ends
    ends_at: Option<Timestamp>,
}

impl Poll {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = Vec::with_capacity(1 + self.options.len() + self.relays.len());

        tags.push(Tag::from_standardized_without_cell(TagStandard::PollType(
            self.r#type,
        )));

        for option in self.options.into_iter() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::PollOption {
                    id: option.id,
                    text: option.text,
                },
            ));
        }

        for url in self.relays.into_iter() {
            tags.push(Tag::relay(url));
        }

        if let Some(timestamp) = self.ends_at {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Ends(
                timestamp,
            )));
        }

        EventBuilder::new(Kind::Poll, self.title).tags(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_type() {
        // Serialization
        assert_eq!(PollType::SingleChoice.as_str(), "singlechoice");
        assert_eq!(PollType::MultipleChoice.as_str(), "multiplechoice");

        // Deserialization
        assert_eq!(
            PollType::from_str("singlechoice").unwrap(),
            PollType::SingleChoice
        );
        assert_eq!(
            PollType::from_str("multiplechoice").unwrap(),
            PollType::MultipleChoice
        );
        assert!(PollType::from_str("unknown").is_err());
    }
}
