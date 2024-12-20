//! NIP35: Torrents
//!
//! This module implements support for sharing BitTorrent metadata and comments through Nostr events.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/35.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use bitcoin::hashes::sha1::Hash as Sha1Hash;

use crate::types::url;
use crate::{EventBuilder, Kind, Tag, TagKind};

/// Represents a file within a torrent.
#[derive(Debug, Clone)]
pub struct TorrentFile {
    /// File name/path
    pub name: String,
    /// File size in bytes
    pub size: u64,
}

/// Torrent metadata
#[derive(Debug, Clone)]
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
    pub trackers: Vec<url::Url>,
    /// Categories (e.g. "video,movie,4k")
    pub categories: Vec<String>,
    /// Additional hashtags
    pub hashtags: Vec<String>,
}

impl Torrent {
    /// Creates a new `EventBuilder` from the given `Torrent` data.
    ///
    /// # Arguments
    ///
    /// * `data` - A `Torrent` struct containing metadata about the torrent.
    ///
    /// # Returns
    ///
    /// An `EventBuilder` initialized with the torrent metadata.
    pub fn torrent(data: Torrent) -> EventBuilder {
        data.to_event_builder()
    }

    /// Converts the torrent metadata into a Nostr event builder.
    pub fn to_event_builder(self) -> EventBuilder {
        let mut tags = Vec::new();

        tags.push(Tag::name(self.title));

        tags.push(Tag::custom(
            TagKind::InfoHash,
            vec![self.info_hash.to_string()],
        ));

        for file in self.files.into_iter() {
            tags.push(Tag::custom(
                TagKind::File,
                vec![file.name, file.size.to_string()],
            ));
        }

        for tracker in self.trackers.into_iter() {
            tags.push(Tag::custom(TagKind::Tracker, vec![tracker]));
        }

        for cat in self.categories.into_iter() {
            tags.push(Tag::custom(
                TagKind::Custom("tcat".into()),
                vec![format!("tcat:{}", cat)],
            ));
        }

        for tag in self.hashtags.into_iter() {
            tags.push(Tag::hashtag(tag));
        }

        EventBuilder::new(Kind::Torrent, self.description).tags(tags)
    }
}
