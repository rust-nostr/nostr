// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

/// Event Kind
#[wasm_bindgen(js_name = Kind)]
pub struct JsKind {
    inner: Kind,
}

impl Deref for JsKind {
    type Target = Kind;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Kind> for JsKind {
    fn from(inner: Kind) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Kind)]
impl JsKind {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: u16) -> Self {
        Self {
            inner: Kind::from_u16(kind),
        }
    }

    #[wasm_bindgen(js_name = fromStd)]
    pub fn from_std(e: JsKindStandard) -> Self {
        Self { inner: e.into() }
    }

    /// Get as 16-bit unsigned integer
    #[wasm_bindgen(js_name = asU16)]
    pub fn as_u16(&self) -> u16 {
        self.inner.as_u16()
    }

    #[wasm_bindgen(js_name = asStd)]
    pub fn as_std(&self) -> Option<JsKindStandard> {
        convert(self.inner)
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn _to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Check if it's regular
    ///
    /// Regular means that event is expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isRegular)]
    pub fn is_regular(&self) -> bool {
        self.inner.is_regular()
    }

    /// Check if it's replaceable
    ///
    /// Replaceable means that, for each combination of `pubkey` and `kind`,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isReplaceable)]
    pub fn is_replaceable(&self) -> bool {
        self.inner.is_replaceable()
    }

    /// Check if it's ephemeral
    ///
    /// Ephemeral means that event is not expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isEphemeral)]
    pub fn is_ephemeral(&self) -> bool {
        self.inner.is_ephemeral()
    }

    /// Check if it's addressable
    ///
    /// Addressable means that, for each combination of `pubkey`, `kind` and the `d` tag's first value,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isAddressable)]
    pub fn is_addressable(&self) -> bool {
        self.inner.is_addressable()
    }

    /// Check if it's a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = isJobRequest)]
    pub fn is_job_request(&self) -> bool {
        self.inner.is_job_request()
    }

    /// Check if it's a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = isJobResult)]
    pub fn is_job_result(&self) -> bool {
        self.inner.is_job_result()
    }
}

/// Standardized kind
#[wasm_bindgen(js_name = KindStandard)]
pub enum JsKindStandard {
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

fn convert(k: Kind) -> Option<JsKindStandard> {
    match k {
        Kind::Metadata => Some(JsKindStandard::Metadata),
        Kind::TextNote => Some(JsKindStandard::TextNote),
        Kind::RecommendRelay | Kind::EncryptedDirectMessage => None,
        Kind::ContactList => Some(JsKindStandard::ContactList),
        Kind::OpenTimestamps => Some(JsKindStandard::OpenTimestamps),
        Kind::EventDeletion => Some(JsKindStandard::EventDeletion),
        Kind::Repost => Some(JsKindStandard::Repost),
        Kind::GenericRepost => Some(JsKindStandard::GenericRepost),
        Kind::Comment => Some(JsKindStandard::Comment),
        Kind::Reaction => Some(JsKindStandard::Reaction),
        Kind::BadgeAward => Some(JsKindStandard::BadgeAward),
        Kind::ChannelCreation => Some(JsKindStandard::ChannelCreation),
        Kind::ChannelMetadata => Some(JsKindStandard::ChannelMetadata),
        Kind::ChannelMessage => Some(JsKindStandard::ChannelMessage),
        Kind::ChannelHideMessage => Some(JsKindStandard::ChannelHideMessage),
        Kind::ChannelMuteUser => Some(JsKindStandard::ChannelMuteUser),
        Kind::PublicChatReserved45
        | Kind::PublicChatReserved46
        | Kind::PublicChatReserved47
        | Kind::PublicChatReserved48
        | Kind::PublicChatReserved49 => None,
        Kind::GitPatch => Some(JsKindStandard::GitPatch),
        Kind::GitIssue => Some(JsKindStandard::GitIssue),
        Kind::GitReply => Some(JsKindStandard::GitReply),
        Kind::GitStatusOpen => Some(JsKindStandard::GitStatusOpen),
        Kind::GitStatusApplied => Some(JsKindStandard::GitStatusApplied),
        Kind::GitStatusClosed => Some(JsKindStandard::GitStatusClosed),
        Kind::GitStatusDraft => Some(JsKindStandard::GitStatusDraft),
        Kind::Label => Some(JsKindStandard::Label),
        Kind::WalletConnectInfo => Some(JsKindStandard::WalletConnectInfo),
        Kind::Reporting => Some(JsKindStandard::Reporting),
        Kind::ZapPrivateMessage => Some(JsKindStandard::ZapPrivateMessage),
        Kind::ZapRequest => Some(JsKindStandard::ZapRequest),
        Kind::ZapReceipt => Some(JsKindStandard::ZapReceipt),
        Kind::MuteList => Some(JsKindStandard::MuteList),
        Kind::PinList => Some(JsKindStandard::PinList),
        Kind::Bookmarks => Some(JsKindStandard::Bookmarks),
        Kind::Communities => Some(JsKindStandard::Communities),
        Kind::PublicChats => Some(JsKindStandard::PublicChats),
        Kind::BlockedRelays => Some(JsKindStandard::BlockedRelays),
        Kind::SearchRelays => Some(JsKindStandard::SearchRelays),
        Kind::SimpleGroups => Some(JsKindStandard::SimpleGroups),
        Kind::Interests => Some(JsKindStandard::Interests),
        Kind::Emojis => Some(JsKindStandard::Emojis),
        Kind::FollowSet => Some(JsKindStandard::FollowSet),
        Kind::RelaySet => Some(JsKindStandard::RelaySet),
        Kind::BookmarkSet => Some(JsKindStandard::BookmarkSet),
        Kind::ArticlesCurationSet => Some(JsKindStandard::ArticlesCurationSet),
        Kind::VideosCurationSet => Some(JsKindStandard::VideosCurationSet),
        Kind::InterestSet => Some(JsKindStandard::InterestSet),
        Kind::EmojiSet => Some(JsKindStandard::EmojiSet),
        Kind::ReleaseArtifactSet => Some(JsKindStandard::ReleaseArtifactSet),
        Kind::RelayList => Some(JsKindStandard::RelayList),
        Kind::Authentication => Some(JsKindStandard::Authentication),
        Kind::WalletConnectRequest => Some(JsKindStandard::WalletConnectRequest),
        Kind::WalletConnectResponse => Some(JsKindStandard::WalletConnectResponse),
        Kind::NostrConnect => Some(JsKindStandard::NostrConnect),
        Kind::LiveEvent => Some(JsKindStandard::LiveEvent),
        Kind::LiveEventMessage => Some(JsKindStandard::LiveEventMessage),
        Kind::ProfileBadges => Some(JsKindStandard::ProfileBadges),
        Kind::BadgeDefinition => Some(JsKindStandard::BadgeDefinition),
        Kind::Seal => Some(JsKindStandard::Seal),
        Kind::GiftWrap => Some(JsKindStandard::GiftWrap),
        Kind::PrivateDirectMessage => Some(JsKindStandard::PrivateDirectMessage),
        Kind::LongFormTextNote => Some(JsKindStandard::LongFormTextNote),
        Kind::GitRepoAnnouncement => Some(JsKindStandard::GitRepoAnnouncement),
        Kind::ApplicationSpecificData => Some(JsKindStandard::ApplicationSpecificData),
        Kind::FileMetadata => Some(JsKindStandard::FileMetadata),
        Kind::HttpAuth => Some(JsKindStandard::HttpAuth),
        Kind::SetStall => Some(JsKindStandard::SetStall),
        Kind::SetProduct => Some(JsKindStandard::SetProduct),
        Kind::JobFeedback => Some(JsKindStandard::JobFeedback),
        Kind::InboxRelays => Some(JsKindStandard::InboxRelays),
        Kind::MlsKeyPackageRelays => Some(JsKindStandard::MlsKeyPackageRelays),
        Kind::MlsKeyPackage => Some(JsKindStandard::MlsKeyPackage),
        Kind::MlsWelcome => Some(JsKindStandard::MlsWelcome),
        Kind::MlsGroupMessage => Some(JsKindStandard::MlsGroupMessage),
        Kind::Torrent => Some(JsKindStandard::Torrent),
        Kind::TorrentComment => Some(JsKindStandard::TorrentComment),
        Kind::PeerToPeerOrder => Some(JsKindStandard::PeerToPeerOrder),
        Kind::RequestToVanish => Some(JsKindStandard::RequestToVanish),
        Kind::UserStatus => Some(JsKindStandard::UserStatus),
        Kind::CashuWallet => Some(JsKindStandard::CashuWallet),
        Kind::CashuWalletUnspentProof => Some(JsKindStandard::CashuWalletUnspentProof),
        Kind::CashuWalletSpendingHistory => Some(JsKindStandard::CashuWalletSpendingHistory),
        Kind::Custom(..) => None,
    }
}

impl From<JsKindStandard> for Kind {
    fn from(value: JsKindStandard) -> Self {
        match value {
            JsKindStandard::Metadata => Self::Metadata,
            JsKindStandard::TextNote => Self::TextNote,
            JsKindStandard::ContactList => Self::ContactList,
            JsKindStandard::OpenTimestamps => Self::OpenTimestamps,
            JsKindStandard::EventDeletion => Self::EventDeletion,
            JsKindStandard::Repost => Self::Repost,
            JsKindStandard::GenericRepost => Self::GenericRepost,
            JsKindStandard::Comment => Self::Comment,
            JsKindStandard::Reaction => Self::Reaction,
            JsKindStandard::BadgeAward => Self::BadgeAward,
            JsKindStandard::ChannelCreation => Self::ChannelCreation,
            JsKindStandard::ChannelMetadata => Self::ChannelMetadata,
            JsKindStandard::ChannelMessage => Self::ChannelMessage,
            JsKindStandard::ChannelHideMessage => Self::ChannelHideMessage,
            JsKindStandard::ChannelMuteUser => Self::ChannelMuteUser,
            JsKindStandard::GitPatch => Self::GitPatch,
            JsKindStandard::GitIssue => Self::GitIssue,
            JsKindStandard::GitReply => Self::GitReply,
            JsKindStandard::GitStatusOpen => Self::GitStatusOpen,
            JsKindStandard::GitStatusApplied => Self::GitStatusApplied,
            JsKindStandard::GitStatusClosed => Self::GitStatusClosed,
            JsKindStandard::GitStatusDraft => Self::GitStatusDraft,
            JsKindStandard::Label => Self::Label,
            JsKindStandard::WalletConnectInfo => Self::WalletConnectInfo,
            JsKindStandard::Reporting => Self::Reporting,
            JsKindStandard::ZapPrivateMessage => Self::ZapPrivateMessage,
            JsKindStandard::ZapRequest => Self::ZapRequest,
            JsKindStandard::ZapReceipt => Self::ZapReceipt,
            JsKindStandard::MuteList => Self::MuteList,
            JsKindStandard::PinList => Self::PinList,
            JsKindStandard::Bookmarks => Self::Bookmarks,
            JsKindStandard::Communities => Self::Communities,
            JsKindStandard::PublicChats => Self::PublicChats,
            JsKindStandard::BlockedRelays => Self::BlockedRelays,
            JsKindStandard::SearchRelays => Self::SearchRelays,
            JsKindStandard::SimpleGroups => Self::SimpleGroups,
            JsKindStandard::Interests => Self::Interests,
            JsKindStandard::Emojis => Self::Emojis,
            JsKindStandard::FollowSet => Self::FollowSet,
            JsKindStandard::RelaySet => Self::RelaySet,
            JsKindStandard::BookmarkSet => Self::BookmarkSet,
            JsKindStandard::ArticlesCurationSet => Self::ArticlesCurationSet,
            JsKindStandard::VideosCurationSet => Self::VideosCurationSet,
            JsKindStandard::InterestSet => Self::InterestSet,
            JsKindStandard::EmojiSet => Self::EmojiSet,
            JsKindStandard::ReleaseArtifactSet => Self::ReleaseArtifactSet,
            JsKindStandard::RelayList => Self::RelayList,
            JsKindStandard::Authentication => Self::Authentication,
            JsKindStandard::WalletConnectRequest => Self::WalletConnectRequest,
            JsKindStandard::WalletConnectResponse => Self::WalletConnectResponse,
            JsKindStandard::NostrConnect => Self::NostrConnect,
            JsKindStandard::LiveEvent => Self::LiveEvent,
            JsKindStandard::LiveEventMessage => Self::LiveEventMessage,
            JsKindStandard::ProfileBadges => Self::ProfileBadges,
            JsKindStandard::BadgeDefinition => Self::BadgeDefinition,
            JsKindStandard::Seal => Self::Seal,
            JsKindStandard::GiftWrap => Self::GiftWrap,
            JsKindStandard::PrivateDirectMessage => Self::PrivateDirectMessage,
            JsKindStandard::LongFormTextNote => Self::LongFormTextNote,
            JsKindStandard::ApplicationSpecificData => Self::ApplicationSpecificData,
            JsKindStandard::GitRepoAnnouncement => Self::GitRepoAnnouncement,
            JsKindStandard::FileMetadata => Self::FileMetadata,
            JsKindStandard::HttpAuth => Self::HttpAuth,
            JsKindStandard::SetStall => Self::SetStall,
            JsKindStandard::SetProduct => Self::SetProduct,
            JsKindStandard::JobFeedback => Self::JobFeedback,
            JsKindStandard::InboxRelays => Self::InboxRelays,
            JsKindStandard::MlsKeyPackageRelays => Self::MlsKeyPackageRelays,
            JsKindStandard::MlsKeyPackage => Self::MlsKeyPackage,
            JsKindStandard::MlsWelcome => Self::MlsWelcome,
            JsKindStandard::MlsGroupMessage => Self::MlsGroupMessage,
            JsKindStandard::Torrent => Self::Torrent,
            JsKindStandard::TorrentComment => Self::TorrentComment,
            JsKindStandard::PeerToPeerOrder => Self::PeerToPeerOrder,
            JsKindStandard::RequestToVanish => Self::RequestToVanish,
            JsKindStandard::UserStatus => Self::UserStatus,
            JsKindStandard::CashuWallet => Self::CashuWallet,
            JsKindStandard::CashuWalletUnspentProof => Self::CashuWalletUnspentProof,
            JsKindStandard::CashuWalletSpendingHistory => Self::CashuWalletSpendingHistory,
        }
    }
}
