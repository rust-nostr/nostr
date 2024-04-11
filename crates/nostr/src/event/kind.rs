// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Kind
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::ParseIntError;
use core::ops::{Add, Range};
use core::str::FromStr;

use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, Serializer};

/// NIP90 - Job request range
pub const NIP90_JOB_REQUEST_RANGE: Range<u64> = 5_000..5_999;
/// NIP90 - Job result range
pub const NIP90_JOB_RESULT_RANGE: Range<u64> = 6_000..6_999;
/// Regular range
pub const REGULAR_RANGE: Range<u64> = 1_000..10_000;
/// Replaceable range
pub const REPLACEABLE_RANGE: Range<u64> = 10_000..20_000;
/// Ephemeral range
pub const EPHEMERAL_RANGE: Range<u64> = 20_000..30_000;
/// Parameterized replaceable range
pub const PARAMETERIZED_REPLACEABLE_RANGE: Range<u64> = 30_000..40_000;

macro_rules! kind_variants {
    ($($name:ident => $value:expr, $doc:expr),* $(,)?) => {
        /// Event [`Kind`]
        #[derive(Debug, Clone, Copy)]
        pub enum Kind {
            $(
                #[doc = $doc]
                $name,
            )*
            /// Represents a job request event (NIP90).
            JobRequest(u16),
            /// Represents a job result event (NIP90).
            JobResult(u16),
            /// Represents a regular event.
            Regular(u16),
            /// Represents a replaceable event.
            Replaceable(u16),
            /// Represents an ephemeral event.
            Ephemeral(u16),
            /// Represents a parameterized replaceable event.
            ParameterizedReplaceable(u16),
            /// Represents a custom event.
            Custom(u64),
        }

        impl From<u64> for Kind {
            fn from(u: u64) -> Self {
                match u {
                    $(
                        $value => Self::$name,
                    )*
                    x if (NIP90_JOB_REQUEST_RANGE).contains(&x) => Self::JobRequest(x as u16),
                    x if (NIP90_JOB_RESULT_RANGE).contains(&x) => Self::JobResult(x as u16),
                    x if (REGULAR_RANGE).contains(&x) => Self::Regular(x as u16),
                    x if (REPLACEABLE_RANGE).contains(&x) => Self::Replaceable(x as u16),
                    x if (EPHEMERAL_RANGE).contains(&x) => Self::Ephemeral(x as u16),
                    x if (PARAMETERIZED_REPLACEABLE_RANGE).contains(&x) => Self::ParameterizedReplaceable(x as u16),
                    x => Self::Custom(x),
                }
            }
        }

        impl From<Kind> for u64 {
            fn from(e: Kind) -> u64 {
                match e {
                    $(
                        Kind::$name => $value,
                    )*
                    Kind::JobRequest(u) => u as u64,
                    Kind::JobResult(u) => u as u64,
                    Kind::Regular(u) => u as u64,
                    Kind::Replaceable(u) => u as u64,
                    Kind::Ephemeral(u) => u as u64,
                    Kind::ParameterizedReplaceable(u) => u as u64,
                    Kind::Custom(u) => u,
                }
            }
        }
    };
}

kind_variants! {
    Metadata => 0, "Metadata (NIP01 and NIP05)",
    TextNote => 1, "Short Text Note (NIP01)",
    RecommendRelay => 2, "Recommend Relay (NIP01 - deprecated)",
    ContactList => 3, "Contacts (NIP02)",
    OpenTimestamps => 1040, "OpenTimestamps Attestations (NIP03)",
    EncryptedDirectMessage => 4, "Encrypted Direct Messages (NIP04)",
    EventDeletion => 5, "Event Deletion (NIP09)",
    Repost => 6, "Repost (NIP18)",
    GenericRepost => 16, "Generic Repost (NIP18)",
    Reaction => 7, "Reaction (NIP25)",
    BadgeAward => 8, "Badge Award (NIP58)",
    ChannelCreation => 40, "Channel Creation (NIP28)",
    ChannelMetadata => 41, "Channel Metadata (NIP28)",
    ChannelMessage => 42, "Channel Message (NIP28)",
    ChannelHideMessage => 43, "Channel Hide Message (NIP28)",
    ChannelMuteUser => 44, "Channel Mute User (NIP28)",
    PublicChatReserved45 => 45, "Public Chat Reserved (NIP28)",
    PublicChatReserved46 => 46, "Public Chat Reserved (NIP28)",
    PublicChatReserved47 => 47, "Public Chat Reserved (NIP28)",
    PublicChatReserved48 => 48, "Public Chat Reserved (NIP28)",
    PublicChatReserved49 => 49, "Public Chat Reserved (NIP28)",
    WalletConnectInfo => 13194, "Wallet Service Info (NIP47)",
    Reporting => 1984, "Reporting (NIP56)",
    Label => 1985, "Label <https://github.com/nostr-protocol/nips/blob/master/32.md>",
    ZapPrivateMessage => 9733, "Zap Private Message (NIP57)",
    ZapRequest => 9734, "Zap Request (NIP57)",
    ZapReceipt => 9735, "Zap Receipt (NIP57)",
    MuteList => 10000, "Mute List <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    PinList => 10001, "Pin List <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Bookmarks => 10003, "Bookmarks <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Communities => 10004, "Communities <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    PublicChats => 10005, "Public Chats <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    BlockedRelays => 10006, "Blocked Relays <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    SearchRelays => 10007, "Search Relays <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    SimpleGroups => 10009, "Simple Groups <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Interests => 10015, "Interests <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Emojis => 10030, "Emojis <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    RelayList => 10002, "Relay List Metadata (NIP65)",
    Authentication => 22242, "Client Authentication (NIP42)",
    WalletConnectRequest => 23194, "Wallet Connect Request (NIP47)",
    WalletConnectResponse => 23195, "Wallet Connect Response (NIP47)",
    NostrConnect => 24133, "Nostr Connect (NIP46)",
    LiveEvent => 30311, "Live Event (NIP53)",
    LiveEventMessage => 1311, "Live Event Message (NIP53)",
    ProfileBadges => 30008, "Profile Badges (NIP58)",
    BadgeDefinition => 30009, "Badge Definition (NIP58)",
    Seal => 13, "Seal (NIP59)",
    GiftWrap => 1059, "Gift Wrap (NIP59)",
    SealedDirect => 14, "GiftWrapped Sealed Direct message",
    SetStall => 30017, "Set stall (NIP15)",
    SetProduct => 30018, "Set product (NIP15)",
    JobFeedback => 7000, "Job Feedback (NIP90)",
    FollowSets => 30000, "Follow Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    RelaySets => 30002, "Relay Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    BookmarkSets => 30003, "Bookmark Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    ArticlesCurationSets => 30004, "Articles Curation Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    VideosCurationSets => 30005, "Videos Curation Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    InterestSets => 30015, "Interest Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    EmojiSets => 30030, "Emoji Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    ReleaseArtifactSets => 30063, "Release Artifact Sets <https://github.com/nostr-protocol/nips/blob/master/51.md>",
    LongFormTextNote => 30023, "Long-form Text Note (NIP23)",
    FileMetadata => 1063, "File Metadata (NIP94)",
    HttpAuth => 27235, "HTTP Auth (NIP98)",
    ApplicationSpecificData => 30078, "Application-specific Data (NIP78)",
}

impl PartialEq<Kind> for Kind {
    fn eq(&self, other: &Kind) -> bool {
        self.as_u64() == other.as_u64()
    }
}

impl Eq for Kind {}

impl PartialOrd for Kind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Kind {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_u64().cmp(&other.as_u64())
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

impl Kind {
    /// Get [`Kind`] as `u32`
    #[inline]
    pub fn as_u32(&self) -> u32 {
        self.as_u64() as u32
    }

    /// Get [`Kind`] as `u64`
    #[inline]
    pub fn as_u64(&self) -> u64 {
        (*self).into()
    }

    /// Get [`Kind`] as `f64`
    #[inline]
    pub fn as_f64(&self) -> f64 {
        self.as_u64() as f64
    }

    /// Check if [`Kind`] is a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[inline]
    pub fn is_job_request(&self) -> bool {
        NIP90_JOB_REQUEST_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[inline]
    pub fn is_job_result(&self) -> bool {
        NIP90_JOB_RESULT_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Regular`
    #[inline]
    pub fn is_regular(&self) -> bool {
        REGULAR_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Replaceable`
    #[inline]
    pub fn is_replaceable(&self) -> bool {
        matches!(self, Kind::Metadata)
            || matches!(self, Kind::ContactList)
            || matches!(self, Kind::ChannelMetadata)
            || REPLACEABLE_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Ephemeral`
    #[inline]
    pub fn is_ephemeral(&self) -> bool {
        EPHEMERAL_RANGE.contains(&self.as_u64())
    }

    /// Check if [`Kind`] is `Parameterized replaceable`
    #[inline]
    pub fn is_parameterized_replaceable(&self) -> bool {
        PARAMETERIZED_REPLACEABLE_RANGE.contains(&self.as_u64())
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u64())
    }
}

impl From<f64> for Kind {
    fn from(kind: f64) -> Self {
        Self::from(kind as u64)
    }
}

impl FromStr for Kind {
    type Err = ParseIntError;

    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind: u64 = kind.parse()?;
        Ok(Self::from(kind))
    }
}

impl Add<u64> for Kind {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        let kind = self.as_u64();
        Kind::from(kind + rhs)
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
        assert_eq!(Kind::ParameterizedReplaceable(30017), Kind::SetStall);
        assert_eq!(Kind::ParameterizedReplaceable(30018), Kind::SetProduct);
    }

    #[test]
    fn test_not_equal_kind() {
        assert_ne!(Kind::Custom(20100), Kind::Custom(2000));
        assert_ne!(Kind::Authentication, Kind::EncryptedDirectMessage);
        assert_ne!(Kind::TextNote, Kind::Custom(2));
    }

    #[test]
    fn test_kind_is_parameterized_replaceable() {
        assert!(Kind::ParameterizedReplaceable(32122).is_parameterized_replaceable());
        assert!(!Kind::ParameterizedReplaceable(1).is_parameterized_replaceable());
    }
}
