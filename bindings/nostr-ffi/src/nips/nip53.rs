// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip53;
use nostr::secp256k1::schnorr::Signature;
use nostr::types::url::UncheckedUrl;
use uniffi::{Enum, Record};

use crate::{ImageDimensions, PublicKey, Timestamp};

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

impl From<LiveEventHost> for nip53::LiveEventHost {
    fn from(value: LiveEventHost) -> Self {
        Self {
            public_key: value.public_key.as_ref().deref().clone(),
            relay_url: value.relay_url.map(UncheckedUrl::from),
            proof: match value.proof {
                Some(sig) => Signature::from_str(&sig).ok(),
                None => None,
            },
        }
    }
}

#[derive(Record)]
pub struct Image {
    pub url: String,
    pub dimensions: Option<Arc<ImageDimensions>>,
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

impl From<LiveEvent> for nip53::LiveEvent {
    fn from(value: LiveEvent) -> Self {
        Self {
            id: value.id,
            title: value.title,
            summary: value.summary,
            image: value
                .image
                .map(|i: Image| (UncheckedUrl::from(i.url), i.dimensions.map(|d| **d))),
            hashtags: value.hashtags,
            streaming: value.streaming.map(UncheckedUrl::from),
            recording: value.recording.map(UncheckedUrl::from),
            starts: value.start.map(|t| **t),
            ends: value.ends.map(|t| **t),
            status: value.status.map(|s| s.into()),
            current_participants: value.current_participants,
            total_participants: value.total_participants,
            relays: value.relays.into_iter().map(UncheckedUrl::from).collect(),
            host: value.host.map(|h| h.into()),
            speakers: value
                .speakers
                .into_iter()
                .map(|s| {
                    (
                        s.public_key.as_ref().deref().clone(),
                        s.url.map(UncheckedUrl::from),
                    )
                })
                .collect(),
            participants: value
                .participants
                .into_iter()
                .map(|s| {
                    (
                        s.public_key.as_ref().deref().clone(),
                        s.url.map(UncheckedUrl::from),
                    )
                })
                .collect(),
        }
    }
}
