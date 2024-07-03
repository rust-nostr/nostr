// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use uniffi::{Enum, Object};

/// Event Kind
#[derive(Debug, PartialEq, Eq, Hash, Object, o2o::o2o)]
#[from_owned(nostr::Kind| return Self { inner: @ })]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Kind {
    inner: nostr::Kind,
}

impl Deref for Kind {
    type Target = nostr::Kind;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Kind {
    #[uniffi::constructor]
    pub fn new(kind: u16) -> Self {
        Self {
            inner: nostr::Kind::from(kind),
        }
    }

    #[uniffi::constructor]
    pub fn from_enum(e: KindEnum) -> Self {
        Self { inner: e.into() }
    }

    /// Get kind as 16-bit unsigned number
    pub fn as_u16(&self) -> u16 {
        self.inner.as_u16()
    }

    /// Get kind as 64-bit unsigned number
    pub fn as_u64(&self) -> u64 {
        self.inner.as_u64()
    }

    pub fn as_enum(&self) -> KindEnum {
        self.inner.into()
    }
}

#[derive(Enum, o2o::o2o)]
#[map_owned(nostr::Kind)]
pub enum KindEnum {
    /// Metadata (NIP01 and NIP05)
    Metadata,
    /// Short Text Note (NIP01)
    TextNote,
    /// Recommend Relay (NIP01 - deprecated)
    RecommendRelay,
    /// Contacts (NIP02)
    ContactList,
    /// OpenTimestamps Attestations (NIP03)
    OpenTimestamps,
    /// Encrypted Direct Messages (NIP04)
    EncryptedDirectMessage,
    /// Event Deletion (NIP09)
    EventDeletion,
    /// Repost (NIP18)
    Repost,
    /// Generic Repost (NIP18)
    GenericRepost,
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
    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    Label,
    /// Wallet Service Info (NIP47)
    WalletConnectInfo,
    /// Reporting (NIP56)
    Reporting,
    /// Zap Private Message (NIP57)
    ZapPrivateMessage,
    /// Zap Request (NIP57)
    ZapRequest,
    /// Zap Receipt (NIP57)
    ZapReceipt,
    /// Mute List
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    MuteList,
    /// Pin List
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    PinList,
    /// Bookmarks
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    Bookmarks,
    /// Communities
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    Communities,
    /// Public Chats
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    PublicChats,
    /// Blocked Relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    BlockedRelays,
    /// Search Relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    SearchRelays,
    /// Simple Groups
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    SimpleGroups,
    /// Interests
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    Interests,
    /// Emojis
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    Emojis,
    /// Follow Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    FollowSet,
    /// Relay Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    RelaySet,
    /// Bookmark Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    BookmarkSet,
    /// Articles Curation Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    ArticlesCurationSet,
    /// Videos Curation Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    VideosCurationSet,
    /// Interest Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    InterestSet,
    /// Emoji Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    EmojiSet,
    /// Release Artifact Set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    ReleaseArtifactSet,
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
    /// Live Event (NIP53)
    LiveEvent,
    /// Live Event Message (NIP53)
    LiveEventMessage,
    /// Profile Badges (NIP58)
    ProfileBadges,
    /// Badge Definition (NIP58)
    BadgeDefinition,
    /// Seal (NIP59)
    Seal,
    /// Gift Wrap (NIP59)
    GiftWrap,
    /// Private Direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    PrivateDirectMessage,
    /// Long-form Text Note (NIP23)
    LongFormTextNote,
    /// Application-specific Data (NIP78)
    ApplicationSpecificData,
    /// File Metadata (NIP94)
    FileMetadata,
    /// HTTP Auth (NIP98)
    HttpAuth,
    /// Set stall (NIP15)
    SetStall,
    /// Set product (NIP15)
    SetProduct,
    /// Job Feedback (NIP90)
    JobFeedback,

    #[o2o(repeat)]
    #[type_hint(as ())]
    
    JobRequest {
        kind: u16,
    },
    JobResult {
        kind: u16,
    },
    Regular {
        kind: u16,
    },
    Replaceable {
        kind: u16,
    },
    Ephemeral {
        kind: u16,
    },
    ParameterizedReplaceable {
        kind: u16,
    },
    Custom {
        kind: u16,
    },
}