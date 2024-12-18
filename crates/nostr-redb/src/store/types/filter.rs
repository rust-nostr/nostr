// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::str::FromStr;

use nostr::{Filter, SingleLetterTag, Timestamp};
use nostr_database::flatbuffers::event_fbs::Fixed32Bytes;

use super::event::DatabaseEvent;

pub struct DatabaseFilter {
    pub ids: HashSet<Fixed32Bytes>,
    pub authors: HashSet<Fixed32Bytes>,
    pub kinds: HashSet<u16>,
    /// Lowercase query
    pub search: Option<String>,
    pub since: Option<Timestamp>,
    pub until: Option<Timestamp>,
    pub generic_tags: BTreeMap<SingleLetterTag, BTreeSet<String>>,
}

impl DatabaseFilter {
    #[inline]
    fn ids_match(&self, event: &DatabaseEvent) -> bool {
        self.ids.is_empty() || self.ids.contains(event.id)
    }

    #[inline]
    fn authors_match(&self, event: &DatabaseEvent) -> bool {
        self.authors.is_empty() || self.authors.contains(event.pubkey)
    }

    #[inline]
    fn tag_match(&self, event: &DatabaseEvent) -> bool {
        if self.generic_tags.is_empty() {
            return true;
        }

        if event.tags.is_empty() {
            return false;
        }

        // TODO: review this code

        // Match
        self.generic_tags.iter().all(|(tag_name, set)| {
            event.tags.iter().filter_map(|t| t.data()).any(|tag| {
                if tag.len() >= 2 {
                    let first: &str = tag.get(0);
                    if let Ok(first) = SingleLetterTag::from_str(first) {
                        if tag_name == &first {
                            let content = tag.get(1);
                            return set.contains(content);
                        }
                    }
                }

                false
            })
        })
    }

    #[inline]
    fn kind_match(&self, event: &DatabaseEvent) -> bool {
        self.kinds.is_empty() || self.kinds.contains(&event.kind)
    }

    #[inline]
    fn search_match(&self, event: &DatabaseEvent) -> bool {
        match &self.search {
            Some(query) => {
                // NOTE: `query` was already converted to lowercase
                let query: &[u8] = query.as_bytes();
                event
                    .content
                    .as_bytes()
                    .windows(query.len())
                    .any(|window| window.eq_ignore_ascii_case(query))
            }
            None => true,
        }
    }

    #[inline]
    pub fn match_event(&self, event: &DatabaseEvent) -> bool {
        self.ids_match(event)
            && self.authors_match(event)
            && self.kind_match(event)
            && self.since.map_or(true, |t| event.created_at >= t)
            && self.until.map_or(true, |t| event.created_at <= t)
            && self.tag_match(event)
            && self.search_match(event)
    }
}

impl From<Filter> for DatabaseFilter {
    fn from(filter: Filter) -> Self {
        Self {
            ids: filter
                .ids
                .map(|ids| {
                    ids.into_iter()
                        .map(|id| Fixed32Bytes::new(id.as_bytes()))
                        .collect()
                })
                .unwrap_or_default(),
            authors: filter
                .authors
                .map(|authors| {
                    authors
                        .into_iter()
                        .map(|pubkey| Fixed32Bytes::new(&pubkey.to_bytes()))
                        .collect()
                })
                .unwrap_or_default(),
            kinds: filter
                .kinds
                .map(|kinds| kinds.into_iter().map(|id| id.as_u16()).collect())
                .unwrap_or_default(),
            search: filter.search.map(|mut s| {
                // Convert to lowercase
                s.make_ascii_lowercase();
                s
            }),
            since: filter.since,
            until: filter.until,
            generic_tags: filter.generic_tags,
        }
    }
}
