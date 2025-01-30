// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP53: Live Activities
//!
//! <https://github.com/nostr-protocol/nips/blob/master/53.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use secp256k1::schnorr::Signature;

use crate::types::{RelayUrl, Url};
use crate::{
    Alphabet, ImageDimensions, PublicKey, SingleLetterTag, Tag, TagKind, TagStandard, Timestamp,
};

/// NIP53 Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Unknown [`LiveEventMarker`]
    UnknownLiveEventMarker(String),
    /// Description missing from event
    DescriptionMissing,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLiveEventMarker(u) => write!(f, "Unknown marker: {u}"),
            Self::DescriptionMissing => write!(f, "Event missing a description"),
        }
    }
}

/// Live Event Marker
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveEventMarker {
    /// Host
    Host,
    /// Speaker
    Speaker,
    /// Participant
    Participant,
}

impl fmt::Display for LiveEventMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Host => write!(f, "Host"),
            Self::Speaker => write!(f, "Speaker"),
            Self::Participant => write!(f, "Participant"),
        }
    }
}

impl FromStr for LiveEventMarker {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Host" => Ok(Self::Host),
            "Speaker" => Ok(Self::Speaker),
            "Participant" => Ok(Self::Participant),
            s => Err(Error::UnknownLiveEventMarker(s.to_string())),
        }
    }
}

/// Live Event Status
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveEventStatus {
    /// Planned
    Planned,
    /// Live
    Live,
    /// Ended
    Ended,
    /// Custom
    Custom(String),
}

impl fmt::Display for LiveEventStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Live => write!(f, "live"),
            Self::Ended => write!(f, "ended"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl<S> From<S> for LiveEventStatus
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        let s: String = s.into();
        match s.as_str() {
            "planned" => Self::Planned,
            "live" => Self::Live,
            "ended" => Self::Ended,
            _ => Self::Custom(s),
        }
    }
}

/// Live Event Host
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveEventHost {
    /// Host public key
    pub public_key: PublicKey,
    /// Host relay URL
    pub relay_url: Option<RelayUrl>,
    /// Host proof
    pub proof: Option<Signature>,
}

/// Live Event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveEvent {
    /// Unique event ID
    pub id: String,
    /// Event title
    pub title: Option<String>,
    /// Event summary
    pub summary: Option<String>,
    /// Event image
    pub image: Option<(Url, Option<ImageDimensions>)>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Steaming URL
    pub streaming: Option<Url>,
    /// Recording URL
    pub recording: Option<Url>,
    /// Starts at
    pub starts: Option<Timestamp>,
    /// Ends at
    pub ends: Option<Timestamp>,
    /// Current status
    pub status: Option<LiveEventStatus>,
    /// Current participants
    pub current_participants: Option<u64>,
    /// Total participants
    pub total_participants: Option<u64>,
    /// Relays
    pub relays: Vec<Url>,
    /// Host
    pub host: Option<LiveEventHost>,
    /// Speakers
    pub speakers: Vec<(PublicKey, Option<RelayUrl>)>,
    /// Participants
    pub participants: Vec<(PublicKey, Option<RelayUrl>)>,
}

impl LiveEvent {
    /// Create a new LiveEvent
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: id.into(),
            title: None,
            summary: None,
            image: None,
            hashtags: Vec::new(),
            streaming: None,
            recording: None,
            starts: None,
            ends: None,
            status: None,
            current_participants: None,
            total_participants: None,
            relays: Vec::new(),
            host: None,
            speakers: Vec::new(),
            participants: Vec::new(),
        }
    }
}

impl From<LiveEvent> for Vec<Tag> {
    fn from(live_event: LiveEvent) -> Self {
        let LiveEvent {
            id,
            title,
            summary,
            image,
            hashtags,
            streaming,
            recording,
            starts,
            ends,
            status,
            current_participants,
            total_participants,
            relays,
            host,
            speakers,
            participants,
        } = live_event;

        let mut tags = Vec::with_capacity(1);

        tags.push(Tag::identifier(id));

        if let Some(title) = title {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Title(
                title,
            )));
        }

        if let Some(summary) = summary {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Summary(
                summary,
            )));
        }

        if let Some(streaming) = streaming {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Streaming(
                streaming,
            )));
        }

        if let Some(status) = status {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::LiveEventStatus(status),
            ));
        }

        if let Some(LiveEventHost {
            public_key,
            relay_url,
            proof,
        }) = host
        {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::PublicKeyLiveEvent {
                    public_key,
                    relay_url,
                    marker: LiveEventMarker::Host,
                    proof,
                },
            ));
        }

        for (public_key, relay_url) in speakers.into_iter() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::PublicKeyLiveEvent {
                    public_key,
                    relay_url,
                    marker: LiveEventMarker::Speaker,
                    proof: None,
                },
            ));
        }

        for (public_key, relay_url) in participants.into_iter() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::PublicKeyLiveEvent {
                    public_key,
                    relay_url,
                    marker: LiveEventMarker::Participant,
                    proof: None,
                },
            ));
        }

        if let Some((image, dim)) = image {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Image(
                image, dim,
            )));
        }

        for hashtag in hashtags.into_iter() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Hashtag(
                hashtag,
            )));
        }

        if let Some(recording) = recording {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Recording(
                recording,
            )));
        }

        if let Some(starts) = starts {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Starts(
                starts,
            )));
        }

        if let Some(ends) = ends {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Ends(ends)));
        }

        if let Some(current_participants) = current_participants {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::CurrentParticipants(current_participants),
            ));
        }

        if let Some(total_participants) = total_participants {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::TotalParticipants(total_participants),
            ));
        }

        if !relays.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Relays(
                relays,
            )));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for LiveEvent {
    type Error = Error;

    fn try_from(tags: Vec<Tag>) -> Result<Self, Self::Error> {
        // Extract content of `d` tag
        let id: &str = tags
            .iter()
            .find(|t| t.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D)))
            .and_then(|t| t.content())
            .ok_or(Error::DescriptionMissing)?;

        let mut live_event = LiveEvent::new(id);

        for tag in tags.into_iter() {
            let Some(tag) = tag.to_standardized() else {
                continue;
            };

            match tag {
                TagStandard::Title(title) => live_event.title = Some(title),
                TagStandard::Summary(summary) => live_event.summary = Some(summary),
                TagStandard::Streaming(url) => live_event.streaming = Some(url),
                TagStandard::LiveEventStatus(status) => live_event.status = Some(status),
                TagStandard::PublicKeyLiveEvent {
                    public_key,
                    relay_url,
                    marker,
                    proof,
                } => match marker {
                    LiveEventMarker::Host => {
                        live_event.host = Some(LiveEventHost {
                            public_key,
                            relay_url,
                            proof,
                        })
                    }
                    LiveEventMarker::Speaker => live_event.speakers.push((public_key, relay_url)),
                    LiveEventMarker::Participant => {
                        live_event.participants.push((public_key, relay_url))
                    }
                },
                TagStandard::Image(image, dim) => live_event.image = Some((image, dim)),
                TagStandard::Hashtag(hashtag) => live_event.hashtags.push(hashtag),
                TagStandard::Recording(url) => live_event.recording = Some(url),
                TagStandard::Starts(starts) => live_event.starts = Some(starts),
                TagStandard::Ends(ends) => live_event.ends = Some(ends),
                TagStandard::CurrentParticipants(n) => live_event.current_participants = Some(n),
                TagStandard::TotalParticipants(n) => live_event.total_participants = Some(n),
                TagStandard::Relays(mut relays) => live_event.relays.append(&mut relays),
                _ => {}
            }
        }

        Ok(live_event)
    }
}
