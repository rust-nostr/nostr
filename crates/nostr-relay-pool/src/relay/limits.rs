// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay limits

/// Limits
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Limits {
    /// Messages limits
    pub messages: MessagesLimits,
    /// Events limits
    pub events: EventsLimits,
}

/// Messages limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessagesLimits {
    /// Maximum size of normalised JSON, in bytes (default: 5_250_000)
    pub max_size: u32,
}

impl Default for MessagesLimits {
    fn default() -> Self {
        Self {
            max_size: 5_250_000,
        }
    }
}

/// Events limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventsLimits {
    /// Maximum size of normalised JSON, in bytes (default: 70_000)
    pub max_size: u32,
    /// Maximum number of tags allowed (default: 2_000)
    pub max_num_tags: u16,
}

impl Default for EventsLimits {
    fn default() -> Self {
        Self {
            max_size: 70_000,
            max_num_tags: 2_000,
        }
    }
}
