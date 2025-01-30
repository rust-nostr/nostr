// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
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
pub const NIP90_JOB_REQUEST_RANGE: Range<u16> = 5_000..6_000;
/// NIP90 - Job result range
pub const NIP90_JOB_RESULT_RANGE: Range<u16> = 6_000..7_000;
/// Regular range
pub const REGULAR_RANGE: Range<u16> = 1_000..10_000;
/// Replaceable range
pub const REPLACEABLE_RANGE: Range<u16> = 10_000..20_000;
/// Ephemeral range
pub const EPHEMERAL_RANGE: Range<u16> = 20_000..30_000;
/// Addressable range
pub const ADDRESSABLE_RANGE: Range<u16> = 30_000..40_000;

macro_rules! kind_variants {
    ($($name:ident => $value:expr, $doc0:expr, $doc1:expr),* $(,)?) => {
        /// Event kind
        ///
        /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
        #[derive(Debug, Clone, Copy)]
        pub enum Kind {
            $(
                #[doc = $doc0]
                #[doc = ""]
                #[doc = $doc1]
                $name,
            )*
            /// Represents a custom event.
            Custom(u16),
        }

        impl From<u16> for Kind {
            fn from(u: u16) -> Self {
                match u {
                    $(
                        $value => Self::$name,
                    )*
                    x => Self::Custom(x),
                }
            }
        }

        impl From<Kind> for u16 {
            fn from(e: Kind) -> u16 {
                match e {
                    $(
                        Kind::$name => $value,
                    )*
                    Kind::Custom(u) => u,
                }
            }
        }
    };
}

kind_variants! {
    Metadata => 0, "Metadata", "<https://github.com/nostr-protocol/nips/blob/master/01.md> and <https://github.com/nostr-protocol/nips/blob/master/05.md>",
    TextNote => 1, "Short Text Note", "<https://github.com/nostr-protocol/nips/blob/master/32.md>",
    RecommendRelay => 2, "Recommend Relay (deprecated)", "",
    ContactList => 3, "Contacts", "<https://github.com/nostr-protocol/nips/blob/master/02.md>",
    OpenTimestamps => 1040, "OpenTimestamps Attestations", "<https://github.com/nostr-protocol/nips/blob/master/03.md>",
    EncryptedDirectMessage => 4, "Encrypted Direct Messages", "<https://github.com/nostr-protocol/nips/blob/master/04.md>",
    EventDeletion => 5, "Event Deletion", "<https://github.com/nostr-protocol/nips/blob/master/09.md>",
    Repost => 6, "Repos", "<https://github.com/nostr-protocol/nips/blob/master/18.md>",
    GenericRepost => 16, "Generic Repos", "<https://github.com/nostr-protocol/nips/blob/master/18.md>",
    Comment => 1111, "Comment", "<https://github.com/nostr-protocol/nips/blob/master/22.md>",
    Reaction => 7, "Reaction", "<https://github.com/nostr-protocol/nips/blob/master/25.md>",
    BadgeAward => 8, "Badge Award", "<https://github.com/nostr-protocol/nips/blob/master/58.md>",
    ChannelCreation => 40, "Channel Creation", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    ChannelMetadata => 41, "Channel Metadata", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    ChannelMessage => 42, "Channel Message", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    ChannelHideMessage => 43, "Channel Hide Message", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    ChannelMuteUser => 44, "Channel Mute User", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    PublicChatReserved45 => 45, "Public Chat Reserved", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    PublicChatReserved46 => 46, "Public Chat Reserved", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    PublicChatReserved47 => 47, "Public Chat Reserved", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    PublicChatReserved48 => 48, "Public Chat Reserved", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    PublicChatReserved49 => 49, "Public Chat Reserved", "<https://github.com/nostr-protocol/nips/blob/master/28.md>",
    MlsKeyPackage => 443, "MLS Key Package", "<https://github.com/nostr-protocol/nips/blob/master/104.md>",
    MlsWelcome => 444, "MLS Welcome", "<https://github.com/nostr-protocol/nips/blob/master/104.md>",
    MlsGroupMessage => 445, "MLS Group Message", "<https://github.com/nostr-protocol/nips/blob/master/104.md>",
    GitPatch => 1617, "Git Patch", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    GitIssue => 1621, "Git Issue", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    GitReply => 1622, "Git Reply", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    GitStatusOpen => 1630, "Open Status of Git Patch or Issue", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    GitStatusApplied => 1631, "Applied / Merged Status of Git Patch or Resolved Status of Git Issue", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    GitStatusClosed => 1632, "Closed Status of Git Patch or Issue", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    GitStatusDraft => 1633, "Draft Status of Git Patch or Issue", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    WalletConnectInfo => 13194, "Wallet Service Info", "<https://github.com/nostr-protocol/nips/blob/master/47.md>",
    Reporting => 1984, "Reporting", "<https://github.com/nostr-protocol/nips/blob/master/56.md>",
    Label => 1985, "Label", "<https://github.com/nostr-protocol/nips/blob/master/32.md>",
    ZapPrivateMessage => 9733, "Zap Private Message ", "<https://github.com/nostr-protocol/nips/blob/master/57.md>",
    ZapRequest => 9734, "Zap Request ", "<https://github.com/nostr-protocol/nips/blob/master/57.md>",
    ZapReceipt => 9735, "Zap Receipt ", "<https://github.com/nostr-protocol/nips/blob/master/57.md>",
    MuteList => 10000, "Mute List", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    PinList => 10001, "Pin List", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Bookmarks => 10003, "Bookmarks", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Communities => 10004, "Communities", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    PublicChats => 10005, "Public Chats", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    BlockedRelays => 10006, "Blocked Relays", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    SearchRelays => 10007, "Search Relays", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    SimpleGroups => 10009, "Simple Groups", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Interests => 10015, "Interests", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    Emojis => 10030, "Emojis", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    InboxRelays => 10050, "Inbox Relays", "<https://github.com/nostr-protocol/nips/blob/master/17.md>",
    MlsKeyPackageRelays => 10051, "MLS Key Package Relays", "<https://github.com/nostr-protocol/nips/blob/master/104.md>",
    RelayList => 10002, "Relay List Metadata", "<https://github.com/nostr-protocol/nips/blob/master/65.md>",
    Authentication => 22242, "Client Authentication", "<https://github.com/nostr-protocol/nips/blob/master/42.md>",
    WalletConnectRequest => 23194, "Wallet Connect Request", "<https://github.com/nostr-protocol/nips/blob/master/47.md>",
    WalletConnectResponse => 23195, "Wallet Connect Response", "<https://github.com/nostr-protocol/nips/blob/master/47.md>",
    NostrConnect => 24133, "Nostr Connect", "<https://github.com/nostr-protocol/nips/blob/master/47.md>",
    LiveEvent => 30311, "Live Event", "<https://github.com/nostr-protocol/nips/blob/master/53.md>",
    LiveEventMessage => 1311, "Live Event Message", "<https://github.com/nostr-protocol/nips/blob/master/53.md>",
    ProfileBadges => 30008, "Profile Badges", "<https://github.com/nostr-protocol/nips/blob/master/58.md>",
    BadgeDefinition => 30009, "Badge Definition", "<https://github.com/nostr-protocol/nips/blob/master/58.md>",
    Seal => 13, "Seal", "<https://github.com/nostr-protocol/nips/blob/master/59.md>",
    GiftWrap => 1059, "Gift Wrap", "<https://github.com/nostr-protocol/nips/blob/master/59.md>",
    PrivateDirectMessage => 14, "Private Direct message", "<https://github.com/nostr-protocol/nips/blob/master/17.md>",
    SetStall => 30017, "Set stall", "<https://github.com/nostr-protocol/nips/blob/master/15.md>",
    SetProduct => 30018, "Set product", "<https://github.com/nostr-protocol/nips/blob/master/15.md>",
    JobFeedback => 7000, "Job Feedback", "<https://github.com/nostr-protocol/nips/blob/master/90.md>",
    FollowSet => 30000, "Follow Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    RelaySet => 30002, "Relay Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    BookmarkSet => 30003, "Bookmark Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    ArticlesCurationSet => 30004, "Articles Curation Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    VideosCurationSet => 30005, "Videos Curation Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    InterestSet => 30015, "Interest Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    EmojiSet => 30030, "Emoji Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    ReleaseArtifactSet => 30063, "Release Artifact Set", "<https://github.com/nostr-protocol/nips/blob/master/51.md>",
    LongFormTextNote => 30023, "Long-form Text Note", "<https://github.com/nostr-protocol/nips/blob/master/23.md>",
    GitRepoAnnouncement => 30617, "Git Repository Announcement", "<https://github.com/nostr-protocol/nips/blob/master/34.md>",
    FileMetadata => 1063, "File Metadata", "<https://github.com/nostr-protocol/nips/blob/master/94.md>",
    HttpAuth => 27235, "HTTP Auth", "<https://github.com/nostr-protocol/nips/blob/master/98.md>",
    ApplicationSpecificData => 30078, "Application-specific Data", "<https://github.com/nostr-protocol/nips/blob/master/78.md>",
    Torrent => 2003, "Torrent", "<https://github.com/nostr-protocol/nips/blob/master/35.md>",
    TorrentComment => 2004, "Torrent Comment", "<https://github.com/nostr-protocol/nips/blob/master/35.md>",
    PeerToPeerOrder => 38383, "Peer-to-peer Order events", "<https://github.com/nostr-protocol/nips/blob/master/69.md>",
}

impl PartialEq for Kind {
    fn eq(&self, other: &Kind) -> bool {
        self.as_u16() == other.as_u16()
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
        self.as_u16().cmp(&other.as_u16())
    }
}

impl Hash for Kind {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_u16().hash(state);
    }
}

impl Kind {
    /// Construct from 16-bit unsigned integer
    #[inline]
    pub fn from_u16(kind: u16) -> Self {
        Self::from(kind)
    }

    /// Get as 16-bit unsigned integer
    #[inline]
    pub fn as_u16(&self) -> u16 {
        (*self).into()
    }

    /// Check if it's regular
    ///
    /// Regular means that event is expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_regular(&self) -> bool {
        let kind: u16 = self.as_u16();

        // Exclude ALL param replaceable and ephemeral
        // Exclude PARTIALLY the replaceable
        if kind > 10_000 {
            return false;
        }

        REGULAR_RANGE.contains(&kind) || !self.is_replaceable()
    }

    /// Check if it's replaceable
    ///
    /// Replaceable means that, for each combination of `pubkey` and `kind`,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn is_replaceable(&self) -> bool {
        matches!(self, Kind::Metadata)
            || matches!(self, Kind::ContactList)
            || matches!(self, Kind::ChannelMetadata)
            || REPLACEABLE_RANGE.contains(&self.as_u16())
    }

    /// Check if it's ephemeral
    ///
    /// Ephemeral means that event is not expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn is_ephemeral(&self) -> bool {
        EPHEMERAL_RANGE.contains(&self.as_u16())
    }

    /// Check if it's addressable
    ///
    /// Addressable means that,
    /// for each combination of `pubkey`, `kind` and the `d` tag's first value,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn is_addressable(&self) -> bool {
        ADDRESSABLE_RANGE.contains(&self.as_u16())
    }

    /// Check if it's a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[inline]
    pub fn is_job_request(&self) -> bool {
        NIP90_JOB_REQUEST_RANGE.contains(&self.as_u16())
    }

    /// Check if it's a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[inline]
    pub fn is_job_result(&self) -> bool {
        NIP90_JOB_RESULT_RANGE.contains(&self.as_u16())
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u16())
    }
}

impl FromStr for Kind {
    type Err = ParseIntError;

    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind: u16 = kind.parse()?;
        Ok(Self::from(kind))
    }
}

impl Add<u16> for Kind {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        let kind: u16 = self.as_u16();
        Kind::from(kind + rhs)
    }
}

impl Serialize for Kind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.as_u16())
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(KindVisitor)
    }
}

struct KindVisitor;

impl Visitor<'_> for KindVisitor {
    type Value = Kind;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a 16-bit unsigned number (0-65535)")
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::Value::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::Value::from(v as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_kind() {
        assert_eq!(Kind::Custom(20100), Kind::from_u16(20100));
        assert_eq!(Kind::TextNote, Kind::Custom(1));
        assert_eq!(Kind::Custom(30017), Kind::SetStall);
        assert_eq!(Kind::Custom(30018), Kind::SetProduct);
    }

    #[test]
    fn test_not_equal_kind() {
        assert_ne!(Kind::Custom(20100), Kind::Custom(2000));
        assert_ne!(Kind::Authentication, Kind::EncryptedDirectMessage);
        assert_ne!(Kind::TextNote, Kind::Custom(2));
    }

    #[test]
    fn test_kind_is_addressable() {
        assert!(Kind::Custom(32122).is_addressable());
        assert!(!Kind::TextNote.is_addressable());
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn parse_ephemeral_kind(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(Kind::from(29_999));
        });
    }

    #[bench]
    pub fn parse_kind(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(Kind::from(0));
        });
    }
}
