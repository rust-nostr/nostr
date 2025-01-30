// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP35: Torrents
//!
//! This module implements support for sharing BitTorrent metadata and comments through nostr events.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/35.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use hashes::sha1::Hash as Sha1Hash;

use crate::types::url::Url;
use crate::{EventBuilder, Kind, Tag, TagKind};

/// Represents a file within a torrent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TorrentFile {
    /// File name/path
    pub name: String,
    /// File size in bytes
    pub size: u64,
}

/// Torrent metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Torrent {
    /// Torrent title
    pub title: String,
    /// Long description
    pub description: String,
    /// BitTorrent info hash
    pub info_hash: Sha1Hash,
    /// Files included in torrent
    pub files: Vec<TorrentFile>,
    /// Tracker URLs
    pub trackers: Vec<Url>,
    /// Categories (e.g. "video,movie,4k")
    pub categories: Vec<String>,
    /// Additional hashtags
    pub hashtags: Vec<String>,
}

impl Torrent {
    /// Converts the torrent metadata into an [`EventBuilder`].
    pub fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = Vec::with_capacity(
            2 + self.files.len()
                + self.trackers.len()
                + self.categories.len()
                + self.hashtags.len(),
        );

        tags.push(Tag::title(self.title));

        tags.push(Tag::custom(TagKind::x(), [self.info_hash.to_string()]));

        for file in self.files.into_iter() {
            tags.push(Tag::custom(
                TagKind::File,
                [file.name, file.size.to_string()],
            ));
        }

        for tracker in self.trackers.into_iter() {
            tags.push(Tag::custom(TagKind::Tracker, [tracker.to_string()]));
        }

        for cat in self.categories.into_iter() {
            tags.push(Tag::custom(TagKind::i(), [format!("tcat:{cat}")]));
        }

        for tag in self.hashtags.into_iter() {
            tags.push(Tag::hashtag(tag));
        }

        EventBuilder::new(Kind::Torrent, self.description).tags(tags)
    }
}
