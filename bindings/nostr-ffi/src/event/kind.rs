// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use uniffi::{Enum, Object};

/// Event Kind
#[derive(Debug, PartialEq, Eq, Hash, Object)]
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

impl From<nostr::Kind> for Kind {
    fn from(inner: nostr::Kind) -> Self {
        Self { inner }
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

#[derive(Enum)]
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
    /// Follow Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    FollowSets,
    /// Relay Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    RelaySets,
    /// Bookmark Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    BookmarkSets,
    /// Articles Curation Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    ArticlesCurationSets,
    /// Videos Curation Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    VideosCurationSets,
    /// Interest Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    InterestSets,
    /// Emoji Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    EmojiSets,
    /// Release Artifact Sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    ReleaseArtifactSets,
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
    /// GiftWrapped Sealed Direct message
    SealedDirect,
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

impl From<nostr::Kind> for KindEnum {
    fn from(value: nostr::Kind) -> Self {
        match value {
            nostr::Kind::Metadata => Self::Metadata,
            nostr::Kind::TextNote => Self::TextNote,
            nostr::Kind::RecommendRelay => Self::RecommendRelay,
            nostr::Kind::ContactList => Self::ContactList,
            nostr::Kind::OpenTimestamps => Self::OpenTimestamps,
            nostr::Kind::EncryptedDirectMessage => Self::EncryptedDirectMessage,
            nostr::Kind::EventDeletion => Self::EventDeletion,
            nostr::Kind::Repost => Self::Repost,
            nostr::Kind::GenericRepost => Self::GenericRepost,
            nostr::Kind::Reaction => Self::Reaction,
            nostr::Kind::BadgeAward => Self::BadgeAward,
            nostr::Kind::ChannelCreation => Self::ChannelCreation,
            nostr::Kind::ChannelMetadata => Self::ChannelMetadata,
            nostr::Kind::ChannelMessage => Self::ChannelMessage,
            nostr::Kind::ChannelHideMessage => Self::ChannelHideMessage,
            nostr::Kind::ChannelMuteUser => Self::ChannelMuteUser,
            nostr::Kind::PublicChatReserved45 => Self::PublicChatReserved45,
            nostr::Kind::PublicChatReserved46 => Self::PublicChatReserved46,
            nostr::Kind::PublicChatReserved47 => Self::PublicChatReserved47,
            nostr::Kind::PublicChatReserved48 => Self::PublicChatReserved48,
            nostr::Kind::PublicChatReserved49 => Self::PublicChatReserved49,
            nostr::Kind::Label => Self::Label,
            nostr::Kind::WalletConnectInfo => Self::WalletConnectInfo,
            nostr::Kind::Reporting => Self::Reporting,
            nostr::Kind::ZapPrivateMessage => Self::ZapPrivateMessage,
            nostr::Kind::ZapRequest => Self::ZapRequest,
            nostr::Kind::ZapReceipt => Self::ZapReceipt,
            nostr::Kind::MuteList => Self::MuteList,
            nostr::Kind::PinList => Self::PinList,
            nostr::Kind::Bookmarks => Self::Bookmarks,
            nostr::Kind::Communities => Self::Communities,
            nostr::Kind::PublicChats => Self::PublicChats,
            nostr::Kind::BlockedRelays => Self::BlockedRelays,
            nostr::Kind::SearchRelays => Self::SearchRelays,
            nostr::Kind::SimpleGroups => Self::SimpleGroups,
            nostr::Kind::Interests => Self::Interests,
            nostr::Kind::Emojis => Self::Emojis,
            nostr::Kind::FollowSets => Self::FollowSets,
            nostr::Kind::RelaySets => Self::RelaySets,
            nostr::Kind::BookmarkSets => Self::BookmarkSets,
            nostr::Kind::ArticlesCurationSets => Self::ArticlesCurationSets,
            nostr::Kind::VideosCurationSets => Self::VideosCurationSets,
            nostr::Kind::InterestSets => Self::InterestSets,
            nostr::Kind::EmojiSets => Self::EmojiSets,
            nostr::Kind::ReleaseArtifactSets => Self::ReleaseArtifactSets,
            nostr::Kind::RelayList => Self::RelayList,
            nostr::Kind::Authentication => Self::Authentication,
            nostr::Kind::WalletConnectRequest => Self::WalletConnectRequest,
            nostr::Kind::WalletConnectResponse => Self::WalletConnectResponse,
            nostr::Kind::NostrConnect => Self::NostrConnect,
            nostr::Kind::LiveEvent => Self::LiveEvent,
            nostr::Kind::LiveEventMessage => Self::LiveEventMessage,
            nostr::Kind::ProfileBadges => Self::ProfileBadges,
            nostr::Kind::BadgeDefinition => Self::BadgeDefinition,
            nostr::Kind::Seal => Self::Seal,
            nostr::Kind::GiftWrap => Self::GiftWrap,
            nostr::Kind::SealedDirect => Self::SealedDirect,
            nostr::Kind::LongFormTextNote => Self::LongFormTextNote,
            nostr::Kind::ApplicationSpecificData => Self::ApplicationSpecificData,
            nostr::Kind::FileMetadata => Self::FileMetadata,
            nostr::Kind::HttpAuth => Self::HttpAuth,
            nostr::Kind::SetStall => Self::SetStall,
            nostr::Kind::SetProduct => Self::SetProduct,
            nostr::Kind::JobFeedback => Self::JobFeedback,
            nostr::Kind::JobRequest(kind) => Self::JobRequest { kind },
            nostr::Kind::JobResult(kind) => Self::JobResult { kind },
            nostr::Kind::Regular(u) => Self::Regular { kind: u },
            nostr::Kind::Replaceable(u) => Self::Replaceable { kind: u },
            nostr::Kind::Ephemeral(u) => Self::Ephemeral { kind: u },
            nostr::Kind::ParameterizedReplaceable(u) => Self::ParameterizedReplaceable { kind: u },
            nostr::Kind::Custom(u) => Self::Custom { kind: u },
        }
    }
}

impl From<KindEnum> for nostr::Kind {
    fn from(value: KindEnum) -> Self {
        match value {
            KindEnum::Metadata => Self::Metadata,
            KindEnum::TextNote => Self::TextNote,
            KindEnum::RecommendRelay => Self::RecommendRelay,
            KindEnum::ContactList => Self::ContactList,
            KindEnum::OpenTimestamps => Self::OpenTimestamps,
            KindEnum::EncryptedDirectMessage => Self::EncryptedDirectMessage,
            KindEnum::EventDeletion => Self::EventDeletion,
            KindEnum::Repost => Self::Repost,
            KindEnum::GenericRepost => Self::GenericRepost,
            KindEnum::Reaction => Self::Reaction,
            KindEnum::BadgeAward => Self::BadgeAward,
            KindEnum::ChannelCreation => Self::ChannelCreation,
            KindEnum::ChannelMetadata => Self::ChannelMetadata,
            KindEnum::ChannelMessage => Self::ChannelMessage,
            KindEnum::ChannelHideMessage => Self::ChannelHideMessage,
            KindEnum::ChannelMuteUser => Self::ChannelMuteUser,
            KindEnum::PublicChatReserved45 => Self::PublicChatReserved45,
            KindEnum::PublicChatReserved46 => Self::PublicChatReserved46,
            KindEnum::PublicChatReserved47 => Self::PublicChatReserved47,
            KindEnum::PublicChatReserved48 => Self::PublicChatReserved48,
            KindEnum::PublicChatReserved49 => Self::PublicChatReserved49,
            KindEnum::Label => Self::Label,
            KindEnum::WalletConnectInfo => Self::WalletConnectInfo,
            KindEnum::Reporting => Self::Reporting,
            KindEnum::ZapPrivateMessage => Self::ZapPrivateMessage,
            KindEnum::ZapRequest => Self::ZapRequest,
            KindEnum::ZapReceipt => Self::ZapReceipt,
            KindEnum::MuteList => Self::MuteList,
            KindEnum::PinList => Self::PinList,
            KindEnum::Bookmarks => Self::Bookmarks,
            KindEnum::Communities => Self::Communities,
            KindEnum::PublicChats => Self::PublicChats,
            KindEnum::BlockedRelays => Self::BlockedRelays,
            KindEnum::SearchRelays => Self::SearchRelays,
            KindEnum::SimpleGroups => Self::SimpleGroups,
            KindEnum::Interests => Self::Interests,
            KindEnum::Emojis => Self::Emojis,
            KindEnum::FollowSets => Self::FollowSets,
            KindEnum::RelaySets => Self::RelaySets,
            KindEnum::BookmarkSets => Self::BookmarkSets,
            KindEnum::ArticlesCurationSets => Self::ArticlesCurationSets,
            KindEnum::VideosCurationSets => Self::VideosCurationSets,
            KindEnum::InterestSets => Self::InterestSets,
            KindEnum::EmojiSets => Self::EmojiSets,
            KindEnum::ReleaseArtifactSets => Self::ReleaseArtifactSets,
            KindEnum::RelayList => Self::RelayList,
            KindEnum::Authentication => Self::Authentication,
            KindEnum::WalletConnectRequest => Self::WalletConnectRequest,
            KindEnum::WalletConnectResponse => Self::WalletConnectResponse,
            KindEnum::NostrConnect => Self::NostrConnect,
            KindEnum::LiveEvent => Self::LiveEvent,
            KindEnum::LiveEventMessage => Self::LiveEventMessage,
            KindEnum::ProfileBadges => Self::ProfileBadges,
            KindEnum::BadgeDefinition => Self::BadgeDefinition,
            KindEnum::Seal => Self::Seal,
            KindEnum::GiftWrap => Self::GiftWrap,
            KindEnum::SealedDirect => Self::SealedDirect,
            KindEnum::LongFormTextNote => Self::LongFormTextNote,
            KindEnum::ApplicationSpecificData => Self::ApplicationSpecificData,
            KindEnum::FileMetadata => Self::FileMetadata,
            KindEnum::HttpAuth => Self::HttpAuth,
            KindEnum::SetStall => Self::SetStall,
            KindEnum::SetProduct => Self::SetProduct,
            KindEnum::JobFeedback => Self::JobFeedback,
            KindEnum::JobRequest { kind } => Self::JobRequest(kind),
            KindEnum::JobResult { kind } => Self::JobResult(kind),
            KindEnum::Regular { kind } => Self::Regular(kind),
            KindEnum::Replaceable { kind } => Self::Replaceable(kind),
            KindEnum::Ephemeral { kind } => Self::Ephemeral(kind),
            KindEnum::ParameterizedReplaceable { kind } => Self::ParameterizedReplaceable(kind),
            KindEnum::Custom { kind } => Self::Custom(kind),
        }
    }
}
