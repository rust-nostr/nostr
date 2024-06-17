// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;

use nostr::{EventId, Url};

/// Send output
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SendOutput {
    /// Set of relay urls to which the message/s was successfully sent
    pub success: HashSet<Url>,
    /// Map of relay urls with related errors where the message/s wasn't sent
    pub failed: HashMap<Url, Option<String>>,
}

impl SendOutput {
    pub(super) fn success(url: Url) -> Self {
        let mut success: HashSet<Url> = HashSet::with_capacity(1);
        success.insert(url);
        Self {
            success,
            failed: HashMap::new(),
        }
    }
}

/// Send event output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendEventOutput {
    /// Event ID
    pub id: EventId,
    /// Set of relay urls to which the message/s was successfully sent
    pub success: HashSet<Url>,
    /// Map of relay urls with related errors where the message/s wasn't sent
    pub failed: HashMap<Url, Option<String>>,
}

impl Deref for SendEventOutput {
    type Target = EventId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl fmt::Display for SendEventOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
