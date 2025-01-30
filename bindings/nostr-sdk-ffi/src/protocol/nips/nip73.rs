// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip73;
use nostr::Url;
use uniffi::Enum;

use crate::error::NostrSdkError;

/// External Content ID
#[derive(Enum)]
pub enum ExternalContentId {
    /// URL
    Url(String),
    /// Hashtag
    Hashtag(String),
    /// Geohash
    Geohash(String),
    /// Book
    Book(String),
    /// Podcast Feed
    PodcastFeed(String),
    /// Podcast Episode
    PodcastEpisode(String),
    /// Podcast Publisher
    PodcastPublisher(String),
    /// Movie
    Movie(String),
    /// Paper
    Paper(String),
}

impl From<nip73::ExternalContentId> for ExternalContentId {
    fn from(content: nip73::ExternalContentId) -> Self {
        match content {
            nip73::ExternalContentId::Url(url) => ExternalContentId::Url(url.to_string()),
            nip73::ExternalContentId::Hashtag(val) => ExternalContentId::Hashtag(val),
            nip73::ExternalContentId::Geohash(val) => ExternalContentId::Geohash(val),
            nip73::ExternalContentId::Book(val) => ExternalContentId::Book(val),
            nip73::ExternalContentId::PodcastFeed(val) => ExternalContentId::PodcastFeed(val),
            nip73::ExternalContentId::PodcastEpisode(val) => ExternalContentId::PodcastEpisode(val),
            nip73::ExternalContentId::PodcastPublisher(val) => {
                ExternalContentId::PodcastPublisher(val)
            }
            nip73::ExternalContentId::Movie(val) => ExternalContentId::Movie(val),
            nip73::ExternalContentId::Paper(val) => ExternalContentId::Paper(val),
        }
    }
}

impl TryFrom<ExternalContentId> for nip73::ExternalContentId {
    type Error = NostrSdkError;

    fn try_from(content: ExternalContentId) -> Result<Self, Self::Error> {
        Ok(match content {
            ExternalContentId::Url(url) => Self::Url(Url::parse(&url)?),
            ExternalContentId::Hashtag(val) => Self::Hashtag(val),
            ExternalContentId::Geohash(val) => Self::Geohash(val),
            ExternalContentId::Book(val) => Self::Book(val),
            ExternalContentId::PodcastFeed(val) => Self::PodcastFeed(val),
            ExternalContentId::PodcastEpisode(val) => Self::PodcastEpisode(val),
            ExternalContentId::PodcastPublisher(val) => Self::PodcastPublisher(val),
            ExternalContentId::Movie(val) => Self::Movie(val),
            ExternalContentId::Paper(val) => Self::Paper(val),
        })
    }
}
