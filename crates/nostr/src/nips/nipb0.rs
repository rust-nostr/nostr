// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIPB0: Web Bookmarks
//!
//! <https://github.com/nostr-protocol/nips/blob/master/B0.md>

use alloc::string::String;
use alloc::vec::Vec;

use crate::{EventBuilder, Kind, Tag, TagStandard, Timestamp};

/// Web Bookmark
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebBookmark {
    /// Description of the web bookmark.
    pub description: String,
    /// URL of the web bookmark.
    pub url: String,
    /// Timestamp when the web bookmark was first published.
    pub published_at: Option<Timestamp>,
    /// Title of the web bookmark.
    pub title: Option<String>,
    /// Hashtags for the web bookmark.
    pub hashtags: Vec<String>,
}

impl WebBookmark {
    /// Create a new web bookmark
    #[inline]
    pub fn new<T>(description: T, url: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            description: description.into(),
            url: url.into(),
            published_at: None,
            title: None,
            hashtags: Vec::new(),
        }
    }

    /// Set the title.
    #[inline]
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Set the timestamp at which the web bookmark was published.
    #[inline]
    pub fn published_at(mut self, timestamp: Timestamp) -> Self {
        self.published_at = Some(timestamp);
        self
    }

    /// Add a hashtag/tag.
    pub fn hashtags<T>(mut self, hashtag: T) -> Self
    where
        T: Into<String>,
    {
        let hashtag = hashtag.into().to_lowercase();
        if !self.hashtags.contains(&hashtag) {
            self.hashtags.push(hashtag);
        }
        self
    }

    /// Convert the web bookmark to an event builder
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = vec![TagStandard::Identifier(self.url).into()];

        let mut add_if_some = |tag: Option<TagStandard>| {
            if let Some(tag) = tag {
                tags.push(tag.into());
            }
        };

        add_if_some(self.published_at.map(TagStandard::PublishedAt));
        add_if_some(self.title.map(TagStandard::Title));

        for hashtag in self.hashtags.into_iter() {
            tags.push(TagStandard::Hashtag(hashtag).into());
        }

        EventBuilder::new(Kind::WebBookmark, self.description).tags(tags)
    }
}
