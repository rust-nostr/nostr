// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP53
//!
//! <https://github.com/nostr-protocol/nips/blob/master/53.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::secp256k1::schnorr::Signature;

use crate::{ImageDimensions, PublicKey, Tag, Timestamp, UncheckedUrl};

/// NIP53 Error
#[derive(Debug)]
pub enum Error {
    /// Unknown [`LiveEventMarker`]
    UnknownLiveEventMarker(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLiveEventMarker(u) => write!(f, "Unknown live event marker: {u}"),
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
pub struct LiveEventHost {
    /// Host public key
    pub public_key: PublicKey,
    /// Host relay URL
    pub relay_url: Option<UncheckedUrl>,
    /// Host proof
    pub proof: Option<Signature>,
}

/// Live Event
pub struct LiveEvent {
    /// Unique event ID
    pub id: String,
    /// Event title
    pub title: Option<String>,
    /// Event summary
    pub summary: Option<String>,
    /// Event image
    pub image: Option<(UncheckedUrl, Option<ImageDimensions>)>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Steaming URL
    pub streaming: Option<UncheckedUrl>,
    /// Recording URL
    pub recording: Option<UncheckedUrl>,
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
    pub relays: Vec<UncheckedUrl>,
    /// Host
    pub host: Option<LiveEventHost>,
    /// Speakers
    pub speakers: Vec<(PublicKey, Option<UncheckedUrl>)>,
    /// Participants
    pub participants: Vec<(PublicKey, Option<UncheckedUrl>)>,
}

impl From<LiveEvent> for Vec<Tag> {
    fn from(live_event: LiveEvent) -> Self {
        let mut tags = Vec::new();

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

        tags.push(Tag::Identifier(id));

        if let Some(title) = title {
            tags.push(Tag::Title(title));
        }

        if let Some(summary) = summary {
            tags.push(Tag::Summary(summary));
        }

        if let Some(streaming) = streaming {
            tags.push(Tag::Streaming(streaming));
        }

        if let Some(status) = status {
            tags.push(Tag::LiveEventStatus(status));
        }

        if let Some(LiveEventHost {
            public_key,
            relay_url,
            proof,
        }) = host
        {
            tags.push(Tag::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker: LiveEventMarker::Host,
                proof,
            });
        }

        for (public_key, relay_url) in speakers.into_iter() {
            tags.push(Tag::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker: LiveEventMarker::Speaker,
                proof: None,
            });
        }

        for (public_key, relay_url) in participants.into_iter() {
            tags.push(Tag::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker: LiveEventMarker::Participant,
                proof: None,
            });
        }

        if let Some((image, dim)) = image {
            tags.push(Tag::Image(image, dim));
        }

        for hashtag in hashtags.into_iter() {
            tags.push(Tag::Hashtag(hashtag));
        }

        if let Some(recording) = recording {
            tags.push(Tag::Recording(recording));
        }

        if let Some(starts) = starts {
            tags.push(Tag::Starts(starts));
        }

        if let Some(ends) = ends {
            tags.push(Tag::Ends(ends));
        }

        if let Some(current_participants) = current_participants {
            tags.push(Tag::CurrentParticipants(current_participants));
        }

        if let Some(total_participants) = total_participants {
            tags.push(Tag::TotalParticipants(total_participants));
        }

        if !relays.is_empty() {
            tags.push(Tag::Relays(relays));
        }

        tags
    }
}
