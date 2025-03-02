// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
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
    pub fn from_std(e: KindStandard) -> Self {
        Self { inner: e.into() }
    }

    /// Get as 16-bit unsigned integer
    pub fn as_u16(&self) -> u16 {
        self.inner.as_u16()
    }

    pub fn as_std(&self) -> Option<KindStandard> {
        convert(self.inner)
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

/// Standardized kind
#[derive(Enum)]
pub enum KindStandard {
    /// Metadata (NIP01 and NIP05)
    Metadata,
    /// Short Text Note (NIP01)
    TextNote,
    /// Contacts (NIP02)
    ContactList,
    /// OpenTimestamps Attestations (NIP03)
    OpenTimestamps,
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
    /// Peer-to-peer Order events
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/69.md>
    PeerToPeerOrder,
    /// Request to Vanish (NIP62)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/62.md>
    RequestToVanish,
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
    /// User Status
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/38.md>
    UserStatus,
    /// Cashu Wallet
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/60.md>
    CashuWallet,
    /// Cashu Wallet Unspent Proof
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/60.md>
    CashuWalletUnspentProof,
    /// Cashu Wallet Spending History
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/60.md>
    CashuWalletSpendingHistory,
}

fn convert(k: nostr::Kind) -> Option<KindStandard> {
    match k {
        nostr::Kind::Metadata => Some(KindStandard::Metadata),
        nostr::Kind::TextNote => Some(KindStandard::TextNote),
        nostr::Kind::RecommendRelay | nostr::Kind::EncryptedDirectMessage => None,
        nostr::Kind::ContactList => Some(KindStandard::ContactList),
        nostr::Kind::OpenTimestamps => Some(KindStandard::OpenTimestamps),
        nostr::Kind::EventDeletion => Some(KindStandard::EventDeletion),
        nostr::Kind::Repost => Some(KindStandard::Repost),
        nostr::Kind::GenericRepost => Some(KindStandard::GenericRepost),
        nostr::Kind::Comment => Some(KindStandard::Comment),
        nostr::Kind::Reaction => Some(KindStandard::Reaction),
        nostr::Kind::BadgeAward => Some(KindStandard::BadgeAward),
        nostr::Kind::ChannelCreation => Some(KindStandard::ChannelCreation),
        nostr::Kind::ChannelMetadata => Some(KindStandard::ChannelMetadata),
        nostr::Kind::ChannelMessage => Some(KindStandard::ChannelMessage),
        nostr::Kind::ChannelHideMessage => Some(KindStandard::ChannelHideMessage),
        nostr::Kind::ChannelMuteUser => Some(KindStandard::ChannelMuteUser),
        nostr::Kind::PublicChatReserved45
        | nostr::Kind::PublicChatReserved46
        | nostr::Kind::PublicChatReserved47
        | nostr::Kind::PublicChatReserved48
        | nostr::Kind::PublicChatReserved49 => None,
        nostr::Kind::GitPatch => Some(KindStandard::GitPatch),
        nostr::Kind::GitIssue => Some(KindStandard::GitIssue),
        nostr::Kind::GitReply => Some(KindStandard::GitReply),
        nostr::Kind::GitStatusOpen => Some(KindStandard::GitStatusOpen),
        nostr::Kind::GitStatusApplied => Some(KindStandard::GitStatusApplied),
        nostr::Kind::GitStatusClosed => Some(KindStandard::GitStatusClosed),
        nostr::Kind::GitStatusDraft => Some(KindStandard::GitStatusDraft),
        nostr::Kind::Label => Some(KindStandard::Label),
        nostr::Kind::WalletConnectInfo => Some(KindStandard::WalletConnectInfo),
        nostr::Kind::Reporting => Some(KindStandard::Reporting),
        nostr::Kind::ZapPrivateMessage => Some(KindStandard::ZapPrivateMessage),
        nostr::Kind::ZapRequest => Some(KindStandard::ZapRequest),
        nostr::Kind::ZapReceipt => Some(KindStandard::ZapReceipt),
        nostr::Kind::MuteList => Some(KindStandard::MuteList),
        nostr::Kind::PinList => Some(KindStandard::PinList),
        nostr::Kind::Bookmarks => Some(KindStandard::Bookmarks),
        nostr::Kind::Communities => Some(KindStandard::Communities),
        nostr::Kind::PublicChats => Some(KindStandard::PublicChats),
        nostr::Kind::BlockedRelays => Some(KindStandard::BlockedRelays),
        nostr::Kind::SearchRelays => Some(KindStandard::SearchRelays),
        nostr::Kind::SimpleGroups => Some(KindStandard::SimpleGroups),
        nostr::Kind::Interests => Some(KindStandard::Interests),
        nostr::Kind::Emojis => Some(KindStandard::Emojis),
        nostr::Kind::FollowSet => Some(KindStandard::FollowSet),
        nostr::Kind::RelaySet => Some(KindStandard::RelaySet),
        nostr::Kind::BookmarkSet => Some(KindStandard::BookmarkSet),
        nostr::Kind::ArticlesCurationSet => Some(KindStandard::ArticlesCurationSet),
        nostr::Kind::VideosCurationSet => Some(KindStandard::VideosCurationSet),
        nostr::Kind::InterestSet => Some(KindStandard::InterestSet),
        nostr::Kind::EmojiSet => Some(KindStandard::EmojiSet),
        nostr::Kind::ReleaseArtifactSet => Some(KindStandard::ReleaseArtifactSet),
        nostr::Kind::RelayList => Some(KindStandard::RelayList),
        nostr::Kind::Authentication => Some(KindStandard::Authentication),
        nostr::Kind::WalletConnectRequest => Some(KindStandard::WalletConnectRequest),
        nostr::Kind::WalletConnectResponse => Some(KindStandard::WalletConnectResponse),
        nostr::Kind::NostrConnect => Some(KindStandard::NostrConnect),
        nostr::Kind::LiveEvent => Some(KindStandard::LiveEvent),
        nostr::Kind::LiveEventMessage => Some(KindStandard::LiveEventMessage),
        nostr::Kind::ProfileBadges => Some(KindStandard::ProfileBadges),
        nostr::Kind::BadgeDefinition => Some(KindStandard::BadgeDefinition),
        nostr::Kind::Seal => Some(KindStandard::Seal),
        nostr::Kind::GiftWrap => Some(KindStandard::GiftWrap),
        nostr::Kind::PrivateDirectMessage => Some(KindStandard::PrivateDirectMessage),
        nostr::Kind::LongFormTextNote => Some(KindStandard::LongFormTextNote),
        nostr::Kind::GitRepoAnnouncement => Some(KindStandard::GitRepoAnnouncement),
        nostr::Kind::ApplicationSpecificData => Some(KindStandard::ApplicationSpecificData),
        nostr::Kind::FileMetadata => Some(KindStandard::FileMetadata),
        nostr::Kind::HttpAuth => Some(KindStandard::HttpAuth),
        nostr::Kind::SetStall => Some(KindStandard::SetStall),
        nostr::Kind::SetProduct => Some(KindStandard::SetProduct),
        nostr::Kind::JobFeedback => Some(KindStandard::JobFeedback),
        nostr::Kind::InboxRelays => Some(KindStandard::InboxRelays),
        nostr::Kind::MlsKeyPackageRelays => Some(KindStandard::MlsKeyPackageRelays),
        nostr::Kind::MlsKeyPackage => Some(KindStandard::MlsKeyPackage),
        nostr::Kind::MlsWelcome => Some(KindStandard::MlsWelcome),
        nostr::Kind::MlsGroupMessage => Some(KindStandard::MlsGroupMessage),
        nostr::Kind::Torrent => Some(KindStandard::Torrent),
        nostr::Kind::TorrentComment => Some(KindStandard::TorrentComment),
        nostr::Kind::PeerToPeerOrder => Some(KindStandard::PeerToPeerOrder),
        nostr::Kind::RequestToVanish => Some(KindStandard::RequestToVanish),
        nostr::Kind::UserStatus => Some(KindStandard::UserStatus),
        nostr::Kind::CashuWallet => Some(KindStandard::CashuWallet),
        nostr::Kind::CashuWalletUnspentProof => Some(KindStandard::CashuWalletUnspentProof),
        nostr::Kind::CashuWalletSpendingHistory => Some(KindStandard::CashuWalletSpendingHistory),
        nostr::Kind::Custom(..) => None,
    }
}

impl From<KindStandard> for nostr::Kind {
    fn from(value: KindStandard) -> Self {
        match value {
            KindStandard::Metadata => Self::Metadata,
            KindStandard::TextNote => Self::TextNote,
            KindStandard::ContactList => Self::ContactList,
            KindStandard::OpenTimestamps => Self::OpenTimestamps,
            KindStandard::EventDeletion => Self::EventDeletion,
            KindStandard::Repost => Self::Repost,
            KindStandard::GenericRepost => Self::GenericRepost,
            KindStandard::Comment => Self::Comment,
            KindStandard::Reaction => Self::Reaction,
            KindStandard::BadgeAward => Self::BadgeAward,
            KindStandard::ChannelCreation => Self::ChannelCreation,
            KindStandard::ChannelMetadata => Self::ChannelMetadata,
            KindStandard::ChannelMessage => Self::ChannelMessage,
            KindStandard::ChannelHideMessage => Self::ChannelHideMessage,
            KindStandard::ChannelMuteUser => Self::ChannelMuteUser,
            KindStandard::GitPatch => Self::GitPatch,
            KindStandard::GitIssue => Self::GitIssue,
            KindStandard::GitReply => Self::GitReply,
            KindStandard::GitStatusOpen => Self::GitStatusOpen,
            KindStandard::GitStatusApplied => Self::GitStatusApplied,
            KindStandard::GitStatusClosed => Self::GitStatusClosed,
            KindStandard::GitStatusDraft => Self::GitStatusDraft,
            KindStandard::Label => Self::Label,
            KindStandard::WalletConnectInfo => Self::WalletConnectInfo,
            KindStandard::Reporting => Self::Reporting,
            KindStandard::ZapPrivateMessage => Self::ZapPrivateMessage,
            KindStandard::ZapRequest => Self::ZapRequest,
            KindStandard::ZapReceipt => Self::ZapReceipt,
            KindStandard::MuteList => Self::MuteList,
            KindStandard::PinList => Self::PinList,
            KindStandard::Bookmarks => Self::Bookmarks,
            KindStandard::Communities => Self::Communities,
            KindStandard::PublicChats => Self::PublicChats,
            KindStandard::BlockedRelays => Self::BlockedRelays,
            KindStandard::SearchRelays => Self::SearchRelays,
            KindStandard::SimpleGroups => Self::SimpleGroups,
            KindStandard::Interests => Self::Interests,
            KindStandard::Emojis => Self::Emojis,
            KindStandard::FollowSet => Self::FollowSet,
            KindStandard::RelaySet => Self::RelaySet,
            KindStandard::BookmarkSet => Self::BookmarkSet,
            KindStandard::ArticlesCurationSet => Self::ArticlesCurationSet,
            KindStandard::VideosCurationSet => Self::VideosCurationSet,
            KindStandard::InterestSet => Self::InterestSet,
            KindStandard::EmojiSet => Self::EmojiSet,
            KindStandard::ReleaseArtifactSet => Self::ReleaseArtifactSet,
            KindStandard::RelayList => Self::RelayList,
            KindStandard::Authentication => Self::Authentication,
            KindStandard::WalletConnectRequest => Self::WalletConnectRequest,
            KindStandard::WalletConnectResponse => Self::WalletConnectResponse,
            KindStandard::NostrConnect => Self::NostrConnect,
            KindStandard::LiveEvent => Self::LiveEvent,
            KindStandard::LiveEventMessage => Self::LiveEventMessage,
            KindStandard::ProfileBadges => Self::ProfileBadges,
            KindStandard::BadgeDefinition => Self::BadgeDefinition,
            KindStandard::Seal => Self::Seal,
            KindStandard::GiftWrap => Self::GiftWrap,
            KindStandard::PrivateDirectMessage => Self::PrivateDirectMessage,
            KindStandard::LongFormTextNote => Self::LongFormTextNote,
            KindStandard::ApplicationSpecificData => Self::ApplicationSpecificData,
            KindStandard::GitRepoAnnouncement => Self::GitRepoAnnouncement,
            KindStandard::FileMetadata => Self::FileMetadata,
            KindStandard::HttpAuth => Self::HttpAuth,
            KindStandard::SetStall => Self::SetStall,
            KindStandard::SetProduct => Self::SetProduct,
            KindStandard::JobFeedback => Self::JobFeedback,
            KindStandard::InboxRelays => Self::InboxRelays,
            KindStandard::MlsKeyPackageRelays => Self::MlsKeyPackageRelays,
            KindStandard::MlsKeyPackage => Self::MlsKeyPackage,
            KindStandard::MlsWelcome => Self::MlsWelcome,
            KindStandard::MlsGroupMessage => Self::MlsGroupMessage,
            KindStandard::Torrent => Self::Torrent,
            KindStandard::TorrentComment => Self::TorrentComment,
            KindStandard::PeerToPeerOrder => Self::PeerToPeerOrder,
            KindStandard::RequestToVanish => Self::RequestToVanish,
            KindStandard::UserStatus => Self::UserStatus,
            KindStandard::CashuWallet => Self::CashuWallet,
            KindStandard::CashuWalletUnspentProof => Self::CashuWalletUnspentProof,
            KindStandard::CashuWalletSpendingHistory => Self::CashuWalletSpendingHistory,
        }
    }
}
