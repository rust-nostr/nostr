// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP88: Polls
//!
//! <https://github.com/nostr-protocol/nips/blob/master/88.md>

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::{Event, EventBuilder, EventId, Kind, RelayUrl, Tag, TagKind, TagStandard, Timestamp};

pub(crate) const ENDS_AT_TAG_KIND_STR: &str = "endsAt";
pub(crate) const ENDS_AT_TAG_KIND: TagKind = TagKind::Custom(Cow::Borrowed(ENDS_AT_TAG_KIND_STR));

/// NIP88 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Unknown poll type
    UnknownPollType,
    /// Unexpected tag
    UnexpectedTag,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPollType => write!(f, "unknown poll type"),
            Self::UnexpectedTag => write!(f, "unexpected tag"),
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
    pub id: String,
    /// Option label
    pub text: String,
}

/// Poll
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Poll {
    /// Poll title
    pub title: String,
    /// Poll type
    pub r#type: PollType,
    /// Poll options
    pub options: Vec<PollOption>,
    /// Relays
    pub relays: Vec<RelayUrl>,
    /// Optionally, when the poll ends
    pub ends_at: Option<Timestamp>,
}

impl Poll {
    /// Parse poll data from an [`Event`].
    pub fn from_event(event: &Event) -> Result<Self, Error> {
        // Search poll type
        let poll_type: PollType = match event.tags.find_standardized(TagKind::PollType) {
            Some(TagStandard::PollType(poll_type)) => *poll_type,
            // Found an unexpected tag.
            Some(..) => return Err(Error::UnexpectedTag),
            // If no valid "polltype" tag is found, the "singlechoice" will be the default.
            None => PollType::SingleChoice,
        };

        // Search poll options
        let options: Vec<PollOption> = event
            .tags
            .filter_standardized(TagKind::Option)
            .filter_map(|tag| match tag {
                TagStandard::PollOption(option) => Some(option.clone()),
                _ => None,
            })
            .collect();

        // Search relays
        let relays: Vec<RelayUrl> = event
            .tags
            .filter_standardized(TagKind::Relay)
            .filter_map(|tag| match tag {
                TagStandard::Relay(url) => Some(url.clone()),
                _ => None,
            })
            .collect();

        // Search ends timestamp
        let ends_at: Option<Timestamp> = match event.tags.find_standardized(ENDS_AT_TAG_KIND) {
            Some(TagStandard::PollEndsAt(timestamp)) => Some(*timestamp),
            Some(..) => return Err(Error::UnexpectedTag),
            None => None,
        };

        Ok(Self {
            title: event.content.clone(),
            r#type: poll_type,
            options,
            relays,
            ends_at,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = Vec::with_capacity(1 + self.options.len() + self.relays.len());

        tags.push(Tag::from_standardized_without_cell(TagStandard::PollType(
            self.r#type,
        )));

        for option in self.options.into_iter() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::PollOption(option),
            ));
        }

        for url in self.relays.into_iter() {
            tags.push(Tag::relay(url));
        }

        if let Some(timestamp) = self.ends_at {
            tags.push(Tag::custom(ENDS_AT_TAG_KIND, [timestamp.to_string()]));
        }

        EventBuilder::new(Kind::Poll, self.title).tags(tags)
    }
}

/// Poll response
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PollResponse {
    /// Single choice
    SingleChoice {
        /// Event ID of poll event
        poll_id: EventId,
        /// Response
        response: String,
    },
    /// Multiple choice
    MultipleChoice {
        /// Event ID of poll event
        poll_id: EventId,
        /// Responses
        responses: Vec<String>,
    },
}

impl PollResponse {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let tags: Vec<Tag> = match self {
            Self::SingleChoice { poll_id, response } => {
                vec![
                    Tag::event(poll_id),
                    Tag::from_standardized_without_cell(TagStandard::PollResponse(response)),
                ]
            }
            Self::MultipleChoice { poll_id, responses } => {
                let mut tags: Vec<Tag> = Vec::with_capacity(1 + responses.len());

                tags.push(Tag::event(poll_id));

                for response in responses.into_iter() {
                    tags.push(Tag::from_standardized_without_cell(
                        TagStandard::PollResponse(response),
                    ));
                }

                tags
            }
        };

        EventBuilder::new(Kind::PollResponse, "").tags(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsonUtil;

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

    #[test]
    fn test_poll_from_event() {
        let event = Event::from_json(r#"{
  "content": "Pineapple on pizza",
  "created_at": 1719888496,
  "id": "9d1b6b9562e66f2ecf35eb0a3c2decc736c47fddb13d6fb8f87185a153ea3634",
  "kind": 1068,
  "pubkey": "dee45a23c4f1d93f3a2043650c5081e4ac14a778e0acbef03de3768e4f81ac7b",
  "sig": "7fa93bf3c430eaef784b0dacc217d3cd5eff1c520e7ef5d961381bc0f014dde6286618048d924808e54d1be03f2f2c2f0f8b5c9c2082a4480caf45a565ca9797",
  "tags": [
    ["option", "qj518h583", "Yay"],
    ["option", "gga6cdnqj", "Nay"],
    ["relay", "wss://relay.damus.io"],
    ["relay", "wss://relay.example.com"],
    ["polltype", "multiplechoice"],
    ["endsAt", "1788888888"]
  ]
}"#).unwrap();

        let poll = Poll::from_event(&event).unwrap();
        assert_eq!(poll.title, "Pineapple on pizza");
        assert_eq!(poll.r#type, PollType::MultipleChoice);
        assert_eq!(
            poll.options,
            vec![
                PollOption {
                    id: "qj518h583".to_string(),
                    text: "Yay".to_string(),
                },
                PollOption {
                    id: "gga6cdnqj".to_string(),
                    text: "Nay".to_string(),
                }
            ]
        );
        assert_eq!(
            poll.relays,
            vec![
                RelayUrl::from_str("wss://relay.damus.io").unwrap(),
                RelayUrl::from_str("wss://relay.example.com").unwrap(),
            ]
        );
        assert_eq!(poll.ends_at, Some(Timestamp::from_secs(1788888888)));
    }

    #[test]
    fn test_poll_from_event_without_poll_type() {
        let event = Event::from_json(r#"{
  "content": "Pineapple on pizza",
  "created_at": 1719888496,
  "id": "9d1b6b9562e66f2ecf35eb0a3c2decc736c47fddb13d6fb8f87185a153ea3634",
  "kind": 1068,
  "pubkey": "dee45a23c4f1d93f3a2043650c5081e4ac14a778e0acbef03de3768e4f81ac7b",
  "sig": "7fa93bf3c430eaef784b0dacc217d3cd5eff1c520e7ef5d961381bc0f014dde6286618048d924808e54d1be03f2f2c2f0f8b5c9c2082a4480caf45a565ca9797",
  "tags": [
    ["option", "qj518h583", "Yay"]
  ]
}"#).unwrap();

        let poll = Poll::from_event(&event).unwrap();
        assert_eq!(poll.title, "Pineapple on pizza");
        assert_eq!(poll.r#type, PollType::SingleChoice);
        assert_eq!(
            poll.options,
            vec![PollOption {
                id: "qj518h583".to_string(),
                text: "Yay".to_string(),
            },]
        );
        assert!(poll.relays.is_empty());
        assert!(poll.ends_at.is_none());
    }

    #[test]
    fn test_poll_from_event_with_empty_poll_type() {
        let event = Event::from_json(r#"{
  "content": "Pineapple on pizza",
  "created_at": 1719888496,
  "id": "9d1b6b9562e66f2ecf35eb0a3c2decc736c47fddb13d6fb8f87185a153ea3634",
  "kind": 1068,
  "pubkey": "dee45a23c4f1d93f3a2043650c5081e4ac14a778e0acbef03de3768e4f81ac7b",
  "sig": "7fa93bf3c430eaef784b0dacc217d3cd5eff1c520e7ef5d961381bc0f014dde6286618048d924808e54d1be03f2f2c2f0f8b5c9c2082a4480caf45a565ca9797",
  "tags": [
    ["option", "qj518h583", "Yay"],
    ["polltype", ""]
  ]
}"#).unwrap();

        let poll = Poll::from_event(&event).unwrap();
        assert_eq!(poll.title, "Pineapple on pizza");
        assert_eq!(poll.r#type, PollType::SingleChoice);
        assert_eq!(
            poll.options,
            vec![PollOption {
                id: "qj518h583".to_string(),
                text: "Yay".to_string(),
            },]
        );
        assert!(poll.relays.is_empty());
        assert!(poll.ends_at.is_none());
    }

    #[test]
    fn test_poll_from_event_with_malformed_polltype_tag() {
        let event = Event::from_json(r#"{
  "content": "Pineapple on pizza",
  "created_at": 1719888496,
  "id": "9d1b6b9562e66f2ecf35eb0a3c2decc736c47fddb13d6fb8f87185a153ea3634",
  "kind": 1068,
  "pubkey": "dee45a23c4f1d93f3a2043650c5081e4ac14a778e0acbef03de3768e4f81ac7b",
  "sig": "7fa93bf3c430eaef784b0dacc217d3cd5eff1c520e7ef5d961381bc0f014dde6286618048d924808e54d1be03f2f2c2f0f8b5c9c2082a4480caf45a565ca9797",
  "tags": [
    ["option", "qj518h583", "Yay"],
    ["polltype"]
  ]
}"#).unwrap();

        let poll = Poll::from_event(&event).unwrap();
        assert_eq!(poll.title, "Pineapple on pizza");
        assert_eq!(poll.r#type, PollType::SingleChoice);
        assert_eq!(
            poll.options,
            vec![PollOption {
                id: "qj518h583".to_string(),
                text: "Yay".to_string(),
            },]
        );
        assert!(poll.relays.is_empty());
        assert!(poll.ends_at.is_none());
    }
}
