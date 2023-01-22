// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;

use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::{Serialize, Serializer};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Kind {
    Metadata,
    TextNote,
    RecommendRelay,
    ContactList,
    EncryptedDirectMessage,
    EventDeletion,
    Repost,
    Reaction,
    ChannelCreation,
    ChannelMetadata,
    ChannelMessage,
    ChannelHideMessage,
    ChannelMuteUser,
    /// Replacabe event (must be between 10000 and <20000)
    Replaceable(u16),
    /// Ephemeral event (must be between 20000 and <30000)
    Ephemeral(u16),
    Custom(u64),
}

impl Kind {
    pub fn as_u64(&self) -> u64 {
        (*self).into()
    }
}

impl From<u64> for Kind {
    fn from(u: u64) -> Self {
        match u {
            0 => Self::Metadata,
            1 => Self::TextNote,
            2 => Self::RecommendRelay,
            3 => Self::ContactList,
            4 => Self::EncryptedDirectMessage,
            5 => Self::EventDeletion,
            6 => Self::Repost,
            7 => Self::Reaction,
            40 => Self::ChannelCreation,
            41 => Self::ChannelMetadata,
            42 => Self::ChannelMessage,
            43 => Self::ChannelHideMessage,
            44 => Self::ChannelMuteUser,
            x if (10_000..20_000).contains(&x) => Self::Replaceable(x as u16),
            x if (20_000..30_000).contains(&x) => Self::Ephemeral(x as u16),
            x => Self::Custom(x),
        }
    }
}

impl From<Kind> for u64 {
    fn from(e: Kind) -> u64 {
        match e {
            Kind::Metadata => 0,
            Kind::TextNote => 1,
            Kind::RecommendRelay => 2,
            Kind::ContactList => 3,
            Kind::EncryptedDirectMessage => 4,
            Kind::EventDeletion => 5,
            Kind::Repost => 6,
            Kind::Reaction => 7,
            Kind::ChannelCreation => 40,
            Kind::ChannelMetadata => 41,
            Kind::ChannelMessage => 42,
            Kind::ChannelHideMessage => 43,
            Kind::ChannelMuteUser => 44,
            Kind::Replaceable(u) => u as u64,
            Kind::Ephemeral(u) => u as u64,
            Kind::Custom(u) => u,
        }
    }
}

impl Serialize for Kind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(From::from(*self))
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(KindVisitor)
    }
}

struct KindVisitor;

impl Visitor<'_> for KindVisitor {
    type Value = Kind;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an unsigned number")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Kind, E>
    where
        E: Error,
    {
        Ok(From::<u64>::from(v))
    }
}
