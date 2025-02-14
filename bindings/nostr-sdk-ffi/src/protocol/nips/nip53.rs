// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip53;
use nostr::secp256k1::schnorr::Signature;
use nostr::types::url::{RelayUrl, Url};
use uniffi::{Enum, Record};

use crate::error::NostrSdkError;
use crate::protocol::key::PublicKey;
use crate::protocol::types::{ImageDimensions, Timestamp};

#[derive(Enum)]
/// Live Event Marker
pub enum LiveEventMarker {
    /// Host
    Host,
    /// Speaker
    Speaker,
    /// Participant
    Participant,
}

impl From<LiveEventMarker> for nip53::LiveEventMarker {
    fn from(value: LiveEventMarker) -> Self {
        match value {
            LiveEventMarker::Host => Self::Host,
            LiveEventMarker::Speaker => Self::Speaker,
            LiveEventMarker::Participant => Self::Participant,
        }
    }
}

impl From<nip53::LiveEventMarker> for LiveEventMarker {
    fn from(value: nip53::LiveEventMarker) -> Self {
        match value {
            nip53::LiveEventMarker::Host => Self::Host,
            nip53::LiveEventMarker::Speaker => Self::Speaker,
            nip53::LiveEventMarker::Participant => Self::Participant,
        }
    }
}

#[derive(Enum)]
pub enum LiveEventStatus {
    Planned,
    Live,
    Ended,
    Custom { custom: String },
}

impl From<LiveEventStatus> for nip53::LiveEventStatus {
    fn from(value: LiveEventStatus) -> Self {
        match value {
            LiveEventStatus::Planned => Self::Planned,
            LiveEventStatus::Live => Self::Live,
            LiveEventStatus::Ended => Self::Ended,
            LiveEventStatus::Custom { custom } => Self::Custom(custom),
        }
    }
}

impl From<nip53::LiveEventStatus> for LiveEventStatus {
    fn from(value: nip53::LiveEventStatus) -> Self {
        match value {
            nip53::LiveEventStatus::Planned => Self::Planned,
            nip53::LiveEventStatus::Live => Self::Live,
            nip53::LiveEventStatus::Ended => Self::Ended,
            nip53::LiveEventStatus::Custom(custom) => Self::Custom { custom },
        }
    }
}

#[derive(Record)]
pub struct LiveEventHost {
    pub public_key: Arc<PublicKey>,
    pub relay_url: Option<String>,
    pub proof: Option<String>,
}

impl TryFrom<LiveEventHost> for nip53::LiveEventHost {
    type Error = NostrSdkError;

    fn try_from(value: LiveEventHost) -> Result<Self, Self::Error> {
        Ok(Self {
            public_key: **value.public_key,
            relay_url: match value.relay_url {
                Some(url) => Some(RelayUrl::parse(&url)?),
                None => None,
            },
            proof: match value.proof {
                Some(sig) => Signature::from_str(&sig).ok(),
                None => None,
            },
        })
    }
}

#[derive(Record)]
pub struct Image {
    pub url: String,
    pub dimensions: Option<ImageDimensions>,
}

#[derive(Record)]
pub struct Person {
    pub public_key: Arc<PublicKey>,
    pub url: Option<String>,
}

#[derive(Record)]
pub struct LiveEvent {
    pub id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub image: Option<Image>,
    pub hashtags: Vec<String>,
    pub streaming: Option<String>,
    pub recording: Option<String>,
    pub start: Option<Arc<Timestamp>>,
    pub ends: Option<Arc<Timestamp>>,
    pub status: Option<LiveEventStatus>,
    pub current_participants: Option<u64>,
    pub total_participants: Option<u64>,
    pub relays: Vec<String>,
    pub host: Option<LiveEventHost>,
    pub speakers: Vec<Person>,
    pub participants: Vec<Person>,
}

impl TryFrom<LiveEvent> for nip53::LiveEvent {
    type Error = NostrSdkError;

    fn try_from(value: LiveEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            title: value.title,
            summary: value.summary,
            image: match value.image {
                Some(i) => Some((Url::parse(&i.url)?, i.dimensions.map(|d| d.into()))),
                None => None,
            },
            hashtags: value.hashtags,
            streaming: match value.streaming {
                Some(url) => Some(Url::parse(&url)?),
                None => None,
            },
            recording: match value.recording {
                Some(url) => Some(Url::parse(&url)?),
                None => None,
            },
            starts: value.start.map(|t| **t),
            ends: value.ends.map(|t| **t),
            status: value.status.map(|s| s.into()),
            current_participants: value.current_participants,
            total_participants: value.total_participants,
            relays: value
                .relays
                .into_iter()
                .filter_map(|u| RelayUrl::parse(&u).ok())
                .collect(),
            host: match value.host {
                Some(h) => Some(h.try_into()?),
                None => None,
            },
            // TODO: propagate error
            speakers: value
                .speakers
                .into_iter()
                .map(|s| (**s.public_key, s.url.and_then(|u| RelayUrl::parse(&u).ok())))
                .collect(),
            participants: value
                .participants
                .into_iter()
                .map(|s| (**s.public_key, s.url.and_then(|u| RelayUrl::parse(&u).ok())))
                .collect(),
        })
    }
}
