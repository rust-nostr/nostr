// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::ops::Deref;

use uniffi::{Enum, Object};

/// Event Kind
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Display, Eq, Hash)]
pub struct Kind {
    inner: nostr::Kind,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
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
            inner: nostr::Kind::from_u16(kind),
        }
    }

    #[uniffi::constructor]
    pub fn from_enum(e: KindEnum) -> Self {
        Self { inner: e.into() }
    }

    /// Get as 16-bit unsigned integer
    pub fn as_u16(&self) -> u16 {
        self.inner.as_u16()
    }

    pub fn as_enum(&self) -> KindEnum {
        self.inner.into()
    }

    /// Check if it's regular
    ///
    /// Regular means that event is expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_regular(&self) -> bool {
        self.inner.is_regular()
    }

    /// Check if it's replaceable
    ///
    /// Replaceable means that, for each combination of `pubkey` and `kind`,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_replaceable(&self) -> bool {
        self.inner.is_replaceable()
    }

    /// Check if it's ephemeral
    ///
    /// Ephemeral means that event is not expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_ephemeral(&self) -> bool {
        self.inner.is_ephemeral()
    }

    /// Check if it's addressable
    ///
    /// Addressable means that, for each combination of `pubkey`, `kind` and the `d` tag's first value,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_addressable(&self) -> bool {
        self.inner.is_addressable()
    }

    /// Check if it's a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_request(&self) -> bool {
        self.inner.is_job_request()
    }

    /// Check if it's a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_result(&self) -> bool {
        self.inner.is_job_result()
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
    /// Comment (NIP22)
    Comment,
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
    /// Git Patch
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitPatch,
    /// Git Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitIssue,
    /// Git Reply
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitReply,
    /// Open Status of Git Patch or Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitStatusOpen,
    /// Applied / Merged Status of Git Patch or Resolved Status of Git Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitStatusApplied,
    /// Closed Status of Git Patch or Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitStatusClosed,
    /// Draft Status of Git Patch or Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitStatusDraft,
    /// Torrent
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/35.md>
    Torrent,
    /// Torrent comment
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/35.md>
    TorrentComment,
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
    /// Inbox Relays (NIP17)
    InboxRelays,
    /// MLS Key Package Relays (NIP104)
    MlsKeyPackageRelays,
    /// MLS Key Package (NIP104)
    MlsKeyPackage,
    /// MLS Welcome (NIP104)
    MlsWelcome,
    /// MLS Group Message (NIP104)
    MlsGroupMessage,
    /// Long-form Text Note (NIP23)
    LongFormTextNote,
    /// Git Repository Announcement
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitRepoAnnouncement,
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
    // TODO: remove this variant
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
            nostr::Kind::Comment => Self::Comment,
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
            nostr::Kind::GitPatch => Self::GitPatch,
            nostr::Kind::GitIssue => Self::GitIssue,
            nostr::Kind::GitReply => Self::GitReply,
            nostr::Kind::GitStatusOpen => Self::GitStatusOpen,
            nostr::Kind::GitStatusApplied => Self::GitStatusApplied,
            nostr::Kind::GitStatusClosed => Self::GitStatusClosed,
            nostr::Kind::GitStatusDraft => Self::GitStatusDraft,
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
            nostr::Kind::FollowSet => Self::FollowSet,
            nostr::Kind::RelaySet => Self::RelaySet,
            nostr::Kind::BookmarkSet => Self::BookmarkSet,
            nostr::Kind::ArticlesCurationSet => Self::ArticlesCurationSet,
            nostr::Kind::VideosCurationSet => Self::VideosCurationSet,
            nostr::Kind::InterestSet => Self::InterestSet,
            nostr::Kind::EmojiSet => Self::EmojiSet,
            nostr::Kind::ReleaseArtifactSet => Self::ReleaseArtifactSet,
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
            nostr::Kind::PrivateDirectMessage => Self::PrivateDirectMessage,
            nostr::Kind::LongFormTextNote => Self::LongFormTextNote,
            nostr::Kind::GitRepoAnnouncement => Self::GitRepoAnnouncement,
            nostr::Kind::ApplicationSpecificData => Self::ApplicationSpecificData,
            nostr::Kind::FileMetadata => Self::FileMetadata,
            nostr::Kind::HttpAuth => Self::HttpAuth,
            nostr::Kind::SetStall => Self::SetStall,
            nostr::Kind::SetProduct => Self::SetProduct,
            nostr::Kind::JobFeedback => Self::JobFeedback,
            nostr::Kind::InboxRelays => Self::InboxRelays,
            nostr::Kind::MlsKeyPackageRelays => Self::MlsKeyPackageRelays,
            nostr::Kind::MlsKeyPackage => Self::MlsKeyPackage,
            nostr::Kind::MlsWelcome => Self::MlsWelcome,
            nostr::Kind::MlsGroupMessage => Self::MlsGroupMessage,
            nostr::Kind::Torrent => Self::Torrent,
            nostr::Kind::TorrentComment => Self::TorrentComment,
            #[allow(deprecated)]
            nostr::Kind::JobRequest(u)
            | nostr::Kind::JobResult(u)
            | nostr::Kind::Regular(u)
            | nostr::Kind::Replaceable(u)
            | nostr::Kind::Ephemeral(u)
            | nostr::Kind::ParameterizedReplaceable(u)
            | nostr::Kind::Custom(u) => Self::Custom { kind: u },
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
            KindEnum::Comment => Self::Comment,
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
            KindEnum::GitPatch => Self::GitPatch,
            KindEnum::GitIssue => Self::GitIssue,
            KindEnum::GitReply => Self::GitReply,
            KindEnum::GitStatusOpen => Self::GitStatusOpen,
            KindEnum::GitStatusApplied => Self::GitStatusApplied,
            KindEnum::GitStatusClosed => Self::GitStatusClosed,
            KindEnum::GitStatusDraft => Self::GitStatusDraft,
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
            KindEnum::FollowSet => Self::FollowSet,
            KindEnum::RelaySet => Self::RelaySet,
            KindEnum::BookmarkSet => Self::BookmarkSet,
            KindEnum::ArticlesCurationSet => Self::ArticlesCurationSet,
            KindEnum::VideosCurationSet => Self::VideosCurationSet,
            KindEnum::InterestSet => Self::InterestSet,
            KindEnum::EmojiSet => Self::EmojiSet,
            KindEnum::ReleaseArtifactSet => Self::ReleaseArtifactSet,
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
            KindEnum::PrivateDirectMessage => Self::PrivateDirectMessage,
            KindEnum::LongFormTextNote => Self::LongFormTextNote,
            KindEnum::ApplicationSpecificData => Self::ApplicationSpecificData,
            KindEnum::GitRepoAnnouncement => Self::GitRepoAnnouncement,
            KindEnum::FileMetadata => Self::FileMetadata,
            KindEnum::HttpAuth => Self::HttpAuth,
            KindEnum::SetStall => Self::SetStall,
            KindEnum::SetProduct => Self::SetProduct,
            KindEnum::JobFeedback => Self::JobFeedback,
            KindEnum::InboxRelays => Self::InboxRelays,
            KindEnum::MlsKeyPackageRelays => Self::MlsKeyPackageRelays,
            KindEnum::MlsKeyPackage => Self::MlsKeyPackage,
            KindEnum::MlsWelcome => Self::MlsWelcome,
            KindEnum::MlsGroupMessage => Self::MlsGroupMessage,
            KindEnum::Torrent => Self::Torrent,
            KindEnum::TorrentComment => Self::TorrentComment,
            KindEnum::Custom { kind } => Self::Custom(kind),
        }
    }
}
