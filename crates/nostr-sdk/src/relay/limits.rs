// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Limits

/// Limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Messages limits
    pub messages: MessagesLimits,
    /// Events limits
    pub events: EventsLimits,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            messages: MessagesLimits {
                max_size: 5_250_000,
            },
            events: EventsLimits { max_size: 70_000 },
        }
    }
}

/// Messages limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessagesLimits {
    /// Maximum size of normalised JSON, in bytes
    pub max_size: u32,
}

/// Events limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventsLimits {
    /// Maximum size of normalised JSON, in bytes
    pub max_size: u32,
    // /// Maximum number of tags allowed
    // pub max_num_tags: u16,
    // Maximum size for tag values, in bytes
    // pub max_tag_val_size: u16,
}
