// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Kind

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::{Serialize, Serializer};

/// Event [`Kind`]
#[derive(Debug, Copy, Clone, Eq, Ord, PartialOrd)]
pub enum Kind {
    /// Metadata (NIP01 and NIP05)
    Metadata,
    /// Short Text Note (NIP01)
    TextNote,
    /// Recommend Relay (NIP01)
    RecommendRelay,
    /// Contacts (NIP02)
    ContactList,
    /// Encrypted Direct Messages (NIP04)
    EncryptedDirectMessage,
    /// Event Deletion (NIP09)
    EventDeletion,
    /// Repost (NIP18)
    Repost,
    /// Reaction (NIP25)
    Reaction,
    /// Channel Creation (NIP28)
    ChannelCreation,
    /// Channel Metadata (NIP28)
    ChannelMetadata,
    /// Channel Message (NIP28)
    ChannelMessage,
    /// Channel Hide Message (NIP28)
    ChannelHideMessage,
    /// Channel Mute User (NIP28)
    ChannelMuteUser,
    /// Public Chat Reserved (NIP28)
    PublicChatReserved45,
    /// Public Chat Reserved (NIP28)
    PublicChatReserved46,
    /// Public Chat Reserved (NIP28)
    PublicChatReserved47,
    /// Public Chat Reserved (NIP28)
    PublicChatReserved48,
    /// Public Chat Reserved (NIP28)
    PublicChatReserved49,
    /// Reporting (NIP56)
    Reporting,
    /// Zap Request (NIP57)
    ZapRequest,
    /// Zap (NIP57)
    Zap,
    /// Client Authentication (NIP42)
    Authentication,
    /// Nostr Connect (NIP46)
    NostrConnect,
    /// Long-form Text Note (NIP23)
    LongFormTextNote,
    /// Relay List Metadata (NIP65)
    RelayList,
    /// Replacabe event (must be between 10000 and <20000)
    Replaceable(u16),
    /// Ephemeral event (must be between 20000 and <30000)
    Ephemeral(u16),
    /// Parameterized Replacabe event (must be between 30000 and <40000)
    ParameterizedReplaceable(u16),
    /// Custom
    Custom(u64),
}

impl Kind {
    /// Get [`Kind`] as `u32`
    pub fn as_u32(&self) -> u32 {
        self.as_u64() as u32
    }

    /// Get [`Kind`] as `u64`
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
            45 => Self::PublicChatReserved45,
            46 => Self::PublicChatReserved46,
            47 => Self::PublicChatReserved47,
            48 => Self::PublicChatReserved48,
            49 => Self::PublicChatReserved49,
            1984 => Self::Reporting,
            9734 => Self::ZapRequest,
            9735 => Self::Zap,
            10002 => Self::RelayList,
            22242 => Self::Authentication,
            24133 => Self::NostrConnect,
            30023 => Self::LongFormTextNote,
            x if (10_000..20_000).contains(&x) => Self::Replaceable(x as u16),
            x if (20_000..30_000).contains(&x) => Self::Ephemeral(x as u16),
            x if (30_000..40_000).contains(&x) => Self::ParameterizedReplaceable(x as u16),
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
            Kind::PublicChatReserved45 => 45,
            Kind::PublicChatReserved46 => 46,
            Kind::PublicChatReserved47 => 47,
            Kind::PublicChatReserved48 => 48,
            Kind::PublicChatReserved49 => 49,
            Kind::Reporting => 1984,
            Kind::ZapRequest => 9734,
            Kind::Zap => 9735,
            Kind::RelayList => 10002,
            Kind::Authentication => 22242,
            Kind::NostrConnect => 24133,
            Kind::LongFormTextNote => 30023,
            Kind::Replaceable(u) => u as u64,
            Kind::Ephemeral(u) => u as u64,
            Kind::ParameterizedReplaceable(u) => u as u64,
            Kind::Custom(u) => u,
        }
    }
}

impl FromStr for Kind {
    type Err = ParseIntError;
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind: u64 = kind.parse()?;
        Ok(Self::from(kind))
    }
}

impl PartialEq<Kind> for Kind {
    fn eq(&self, other: &Kind) -> bool {
        self.as_u64() == other.as_u64()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_kind() {
        assert_eq!(Kind::Custom(20100), Kind::Custom(20100));
        assert_eq!(Kind::Custom(20100), Kind::Ephemeral(20100));
        assert_eq!(Kind::TextNote, Kind::Custom(1));
    }

    #[test]
    fn test_not_equal_kind() {
        assert_ne!(Kind::Custom(20100), Kind::Custom(2000));
        assert_ne!(Kind::Authentication, Kind::EncryptedDirectMessage);
        assert_ne!(Kind::TextNote, Kind::Custom(2));
    }
}
