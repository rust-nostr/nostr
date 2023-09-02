// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP53
//!
//! <https://github.com/nostr-protocol/nips/blob/master/53.md>

use alloc::string::String;
use alloc::vec::Vec;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::XOnlyPublicKey;

use crate::event::tag::{LiveEventMarker, LiveEventStatus};
use crate::{ImageDimensions, Tag, Timestamp, UncheckedUrl};

/// Live Event Host
pub struct LiveEventHost {
    /// Host public key
    pub public_key: XOnlyPublicKey,
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
    pub speakers: Vec<(XOnlyPublicKey, Option<UncheckedUrl>)>,
    /// Participants
    pub participants: Vec<(XOnlyPublicKey, Option<UncheckedUrl>)>,
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
            tags.push(Tag::Status(status));
        }

        if let Some(LiveEventHost {
            public_key,
            relay_url,
            proof,
        }) = host
        {
            tags.push(Tag::PubKeyLiveEvent {
                pk: public_key,
                relay_url,
                marker: LiveEventMarker::Host,
                proof,
            });
        }

        for (pk, relay_url) in speakers.into_iter() {
            tags.push(Tag::PubKeyLiveEvent {
                pk,
                relay_url,
                marker: LiveEventMarker::Speaker,
                proof: None,
            });
        }

        for (pk, relay_url) in participants.into_iter() {
            tags.push(Tag::PubKeyLiveEvent {
                pk,
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
