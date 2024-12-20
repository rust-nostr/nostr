//! NIP35: Torrents
//!
//! This module implements support for sharing BitTorrent metadata and comments through Nostr events.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/35.md>

use alloc::string::String;
use alloc::vec::Vec;

use crate::alloc::string::ToString;
use crate::{EventBuilder, EventId, Kind, RelayUrl, Tag, TagKind};

#[derive(Debug, Clone)]
/// Represents a file within a torrent.
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
    pub info_hash: String,
    /// Files included in torrent
    pub files: Vec<TorrentFile>,
    /// Tracker URLs
    pub trackers: Vec<String>,
    /// Categories (e.g. "video,movie,4k")
    pub categories: Vec<String>,
    /// Additional hashtags
    pub hashtags: Vec<String>,
}

impl Torrent {
    /// Converts the torrent metadata into a Nostr event builder.

    pub fn to_event_builder(self) -> EventBuilder {
        let mut tags = Vec::new();

        tags.push(Tag::name(self.title));

        if self.info_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            tags.push(Tag::custom(
                TagKind::Custom("x".into()),
                vec![self.info_hash],
            ));
        } else {
            panic!("Invalid info hash: not a valid hex string");
        }

        for file in self.files {
            tags.push(Tag::custom(
                TagKind::Custom("file".into()),
                vec![file.name, file.size.to_string()],
            ));
        }

        for tracker in self.trackers {
            tags.push(Tag::custom(
                TagKind::Custom("tracker".into()),
                vec![tracker],
            ));
        }

        for cat in self.categories {
            tags.push(Tag::custom(
                TagKind::Custom("tcat".into()),
                vec![format!("tcat:{}", cat)],
            ));
        }

        for tag in self.hashtags {
            tags.push(Tag::hashtag(tag));
        }

        EventBuilder::new(Kind::Torrent, self.description).tags(tags)
    }
}

/// Torrent comment
#[derive(Debug, Clone)]
pub struct TorrentComment {
    /// The torrent event being commented on
    pub torrent_event: EventId,
    /// Comment content
    pub content: String,
    /// Optional relay URL where the original torrent can be found
    pub relay_url: Option<RelayUrl>,
}

impl TorrentComment {
    /// Converts the torrent comment into a Nostr event builder.
    pub fn to_event_builder(self) -> EventBuilder {
        let mut tags = vec![Tag::event(self.torrent_event)];

        if let Some(relay_url) = self.relay_url {
            tags.push(Tag::custom(
                TagKind::Custom("r".into()),
                vec![relay_url.to_string()],
            ));
        }

        EventBuilder::new(Kind::TorrentComment, self.content).tags(tags)
    }
}
