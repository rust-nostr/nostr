// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Kind

use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::ParseIntError;
use core::ops::Range;
use core::str::FromStr;

use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, Serializer};

/// Regular range
pub const REGULAR_RANGE: Range<u64> = 1_000..10_000;
/// Replaceable range
pub const REPLACEABLE_RANGE: Range<u64> = 10_000..20_000;
/// Ephemeral range
pub const EPHEMERAL_RANGE: Range<u64> = 20_000..30_000;
/// Parameterized replaceable range
pub const PARAMETERIZED_REPLACEABLE_RANGE: Range<u64> = 30_000..40_000;

/// Event [`Kind`]
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
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
    /// Badge Award (NIP58)
    BadgeAward,
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
    /// Wallet Service Info (NIP47)
    WalletConnectInfo,
    /// Reporting (NIP56)
    Reporting,
    /// Zap Request (NIP57)
    ZapRequest,
    /// Zap Receipt (NIP57)
    ZapReceipt,
    /// Mute List (NIP51)
    MuteList,
    /// Pin List (NIP51)
    PinList,
    /// Relay List Metadata (NIP65)
    RelayList,
    /// Client Authentication (NIP42)
    Authentication,
    /// Wallet Connect Request (NIP47)
    WalletConnectRequest,
    /// Wallet Connect Response (NIP47)
    WalletConnectResponse,
    /// Nostr Connect (NIP46)
    NostrConnect,
    /// Categorized People List (NIP51)
    CategorizedPeopleList,
    /// Categorized Bookmark List (NIP51)
    CategorizedBookmarkList,
    /// Live Event (NIP53)
    LiveEvent,
    /// Live Event Message (NIP53)
    LiveEventMessage,
    /// Profile Badges (NIP58)
    ProfileBadges,
    /// Badge Definition (NIP58)
    BadgeDefinition,
    /// Long-form Text Note (NIP23)
    LongFormTextNote,
    /// Application-specific Data (NIP78)
    ApplicationSpecificData,
    /// File Metadata (NIP94)
    FileMetadata,
    /// HTTP Auth (NIP98)
    HttpAuth,
    /// Regular Events (must be between 1000 and <=9999)
    Regular(u16),
    /// Replaceable event (must be between 10000 and <20000)
    Replaceable(u16),
    /// Ephemeral event (must be between 20000 and <30000)
    Ephemeral(u16),
    /// Parameterized replaceable event (must be between 30000 and <40000)
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

    /// Check if [`Kind`] is `Regular`
    pub fn is_regular(&self) -> bool {
        REGULAR_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Replaceable`
    pub fn is_replaceable(&self) -> bool {
        REPLACEABLE_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Ephemeral`
    pub fn is_ephemeral(&self) -> bool {
        EPHEMERAL_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Parameterized replaceable`
    pub fn is_parameterized_replaceable(&self) -> bool {
        PARAMETERIZED_REPLACEABLE_RANGE.contains(&self.as_u64())
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u64())
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
            8 => Self::BadgeAward,
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
            13194 => Self::WalletConnectInfo,
            1984 => Self::Reporting,
            9734 => Self::ZapRequest,
            9735 => Self::ZapReceipt,
            10000 => Self::MuteList,
            10001 => Self::PinList,
            10002 => Self::RelayList,
            22242 => Self::Authentication,
            23194 => Self::WalletConnectRequest,
            23195 => Self::WalletConnectResponse,
            24133 => Self::NostrConnect,
            30000 => Self::CategorizedPeopleList,
            30001 => Self::CategorizedBookmarkList,
            30311 => Self::LiveEvent,
            1311 => Self::LiveEventMessage,
            30008 => Self::ProfileBadges,
            30009 => Self::BadgeDefinition,
            30023 => Self::LongFormTextNote,
            30078 => Self::ApplicationSpecificData,
            1063 => Self::FileMetadata,
            27235 => Self::HttpAuth,
            x if (REGULAR_RANGE).contains(&x) => Self::Regular(x as u16),
            x if (REPLACEABLE_RANGE).contains(&x) => Self::Replaceable(x as u16),
            x if (EPHEMERAL_RANGE).contains(&x) => Self::Ephemeral(x as u16),
            x if (PARAMETERIZED_REPLACEABLE_RANGE).contains(&x) => {
                Self::ParameterizedReplaceable(x as u16)
            }
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
            Kind::BadgeAward => 8,
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
            Kind::WalletConnectInfo => 13194,
            Kind::Reporting => 1984,
            Kind::ZapRequest => 9734,
            Kind::ZapReceipt => 9735,
            Kind::MuteList => 10000,
            Kind::PinList => 10001,
            Kind::RelayList => 10002,
            Kind::Authentication => 22242,
            Kind::WalletConnectRequest => 23194,
            Kind::WalletConnectResponse => 23195,
            Kind::NostrConnect => 24133,
            Kind::CategorizedPeopleList => 30000,
            Kind::CategorizedBookmarkList => 30001,
            Kind::LiveEvent => 30311,
            Kind::LiveEventMessage => 1311,
            Kind::ProfileBadges => 30008,
            Kind::BadgeDefinition => 30009,
            Kind::LongFormTextNote => 30023,
            Kind::ApplicationSpecificData => 30078,
            Kind::FileMetadata => 1063,
            Kind::HttpAuth => 27235,
            Kind::Regular(u) => u as u64,
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

impl Hash for Kind {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_u64().hash(state);
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
