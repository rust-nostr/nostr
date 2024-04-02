// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay limits

/// Relay limits
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelayLimits {
    /// Message limits
    pub messages: RelayMessageLimits,
    /// Event limits
    pub events: RelayEventLimits,
}

impl RelayLimits {
    /// Disable all limits
    #[inline]
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
    /// Maximum size of normalised JSON, in bytes (default: 5_250_000)
    pub max_size: Option<u32>,
}

impl Default for RelayMessageLimits {
    fn default() -> Self {
        Self {
            max_size: Some(5_250_000),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayEventLimits {
    /// Maximum size of normalised JSON, in bytes (default: 70_000)
    pub max_size: Option<u32>,
    /// Maximum number of tags allowed (default: 2_000)
    pub max_num_tags: Option<u16>,
}

impl Default for RelayEventLimits {
    fn default() -> Self {
        Self {
            max_size: Some(70_000),
            max_num_tags: Some(2_000),
        }
    }
}

impl RelayEventLimits {
    /// Disable all limits
    #[inline]
    pub fn disable() -> Self {
        Self {
            max_size: None,
            max_num_tags: None,
        }
    }
}
