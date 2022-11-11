// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Eq, PartialEq, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
#[serde(untagged)]
pub enum Kind {
    Base(KindBase),
    Custom(u16),
}
