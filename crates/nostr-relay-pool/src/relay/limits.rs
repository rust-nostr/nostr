// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay limits

use std::collections::HashMap;

use nostr::Kind;

use super::constants::{MAX_CONTACT_LIST_EVENT_SIZE, MAX_EVENT_SIZE, MAX_MESSAGE_SIZE};

/// Relay limits
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelayLimits {
    /// Message limits
    pub messages: RelayMessageLimits,
    /// Event limits
    pub events: RelayEventLimits,
}

impl RelayLimits {
    /// Disable all limits
    pub fn disable() -> Self {
        Self {
            messages: RelayMessageLimits::disable(),
            events: RelayEventLimits::disable(),
        }
    }
}

/// Messages limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayMessageLimits {
    /// Maximum size of normalised JSON, in bytes (default: [`MAX_MESSAGE_SIZE`])
    pub max_size: Option<u32>,
}

impl Default for RelayMessageLimits {
    fn default() -> Self {
        Self {
            max_size: Some(MAX_MESSAGE_SIZE),
        }
    }
}

impl RelayMessageLimits {
    /// Disable all limits
    #[inline]
    pub fn disable() -> Self {
        Self { max_size: None }
    }
}

/// Events limits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayEventLimits {
    /// Maximum size of normalised JSON, in bytes (default: [`MAX_EVENT_SIZE`])
    pub max_size: Option<u32>,
    /// Maximum size of normalized JSON per [Kind], in bytes.
    pub max_size_per_kind: HashMap<Kind, Option<u32>>,
    /// Maximum number of tags allowed (default: 2_000)
    pub max_num_tags: Option<u16>,
    /// Maximum number of tags allowed per [Kind].
    pub max_num_tags_per_kind: HashMap<Kind, Option<u16>>,
}

impl Default for RelayEventLimits {
    fn default() -> Self {
        let mut max_size_per_kind: HashMap<Kind, Option<u32>> = HashMap::with_capacity(1);
        max_size_per_kind.insert(Kind::ContactList, Some(MAX_CONTACT_LIST_EVENT_SIZE));

        let mut max_num_tags_per_kind: HashMap<Kind, Option<u16>> = HashMap::with_capacity(1);
        max_num_tags_per_kind.insert(Kind::ContactList, Some(10_000));

        Self {
            max_size: Some(MAX_EVENT_SIZE),
            max_size_per_kind,
            max_num_tags: Some(2_000),
            max_num_tags_per_kind,
        }
    }
}

impl RelayEventLimits {
    /// Disable all limits
    pub fn disable() -> Self {
        Self {
            max_size: None,
            max_size_per_kind: HashMap::new(),
            max_num_tags: None,
            max_num_tags_per_kind: HashMap::new(),
        }
    }

    /// Add/Edit max size per [Kind]
    pub fn set_max_size_per_kind(mut self, kind: Kind, max_size: Option<u32>) -> Self {
        self.max_size_per_kind.insert(kind, max_size);
        self
    }

    /// Add/Edit max number of tags per [Kind]
    pub fn set_max_num_tags_per_kind(mut self, kind: Kind, max_num_tags: Option<u16>) -> Self {
        self.max_num_tags_per_kind.insert(kind, max_num_tags);
        self
    }

    /// Get max size for [Kind]
    ///
    /// Fallback to `max_size` if no limit is specified for [Kind]
    pub fn get_max_size(&self, kind: &Kind) -> Option<u32> {
        match self.max_size_per_kind.get(kind).copied() {
            Some(limit) => limit,
            None => self.max_size,
        }
    }

    /// Get max number of tags allowed for [Kind]
    ///
    /// Fallback to `max_num_tags` if no limit is specified for [Kind]
    pub fn get_max_num_tags(&self, kind: &Kind) -> Option<u16> {
        match self.max_num_tags_per_kind.get(kind).copied() {
            Some(limit) => limit,
            None => self.max_num_tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_limits_get_max_size() {
        let limits = RelayLimits::default();

        assert_eq!(
            limits.events.get_max_size(&Kind::TextNote),
            Some(MAX_EVENT_SIZE)
        );
        assert_eq!(
            limits.events.get_max_size(&Kind::ContactList),
            Some(MAX_CONTACT_LIST_EVENT_SIZE)
        );
    }
}
