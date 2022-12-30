// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum KindBase {
    Metadata = 0,
    TextNote = 1,
    RecommendRelay = 2,
    ContactList = 3,
    EncryptedDirectMessage = 4,
    EventDeletion = 5,
    Boost = 6,
    Reaction = 7,
    ChannelCreation = 40,
    ChannelMetadata = 41,
    ChannelMessage = 42,
    ChannelHideMessage = 43,
    ChannelMuteUser = 44,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Kind {
    Base(KindBase),
    Custom(u64),
}

impl Kind {
    pub fn as_u64(&self) -> u64 {
        match *self {
            Self::Base(kind) => kind as u64,
            Self::Custom(kind) => kind,
        }
    }
}
