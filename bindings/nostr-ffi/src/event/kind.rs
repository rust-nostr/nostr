// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

pub struct Kind {
    inner: nostr::Kind
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

impl From<Kind> for nostr::Kind {
    fn from(kind: Kind) -> Self {
        kind.inner
    }
}

impl Kind {
    pub fn new(kind: u64) -> Self {
        Self {
            inner: nostr::Kind::from(kind)
        }
    }

    pub fn from_enum(e: KindEnum) -> Self {
        Self {
            inner: e.into()
        }
    }

    pub fn as_u64(&self) -> u64 {
        self.inner.as_u64()
    }

    pub fn as_enum(&self) -> KindEnum {
        self.inner.into()
    }
}

pub enum KindEnum {
    MetadataK,
    TextNote,
    RecommendRelay,
    ContactList,
    EncryptedDirectMessage,
    EventDeletion,
    Repost,
    Reaction,
    BadgeAward,
    ChannelCreation,
    ChannelMetadata,
    ChannelMessage,
    ChannelHideMessage,
    ChannelMuteUser,
    PublicChatReserved45,
    PublicChatReserved46,
    PublicChatReserved47,
    PublicChatReserved48,
    PublicChatReserved49,
    WalletConnectInfo,
    Reporting,
    ZapRequest,
    ZapReceipt,
    MuteList,
    PinList,
    RelayList,
    Authentication,
    WalletConnectRequest,
    WalletConnectResponse,
    NostrConnect,
    CategorizedPeopleList,
    CategorizedBookmarkList,
    LiveEvent,
    LiveEventMessage,
    ProfileBadges,
    BadgeDefinition,
    LongFormTextNote,
    ApplicationSpecificData,
    FileMetadataK,
    HttpAuth,
    Regular { kind: u16 },
    Replaceable { kind: u16 },
    Ephemeral { kind: u16 },
    ParameterizedReplaceable { kind: u16 },
    Custom { kind: u64 },
}

impl From<nostr::Kind> for KindEnum {
    fn from(value: nostr::Kind) -> Self {
        match value {
            nostr::Kind::Metadata => Self::MetadataK,
            nostr::Kind::TextNote => Self::TextNote,
            nostr::Kind::RecommendRelay => Self::RecommendRelay,
            nostr::Kind::ContactList => Self::ContactList,
            nostr::Kind::EncryptedDirectMessage => Self::EncryptedDirectMessage,
            nostr::Kind::EventDeletion => Self::EventDeletion,
            nostr::Kind::Repost => Self::Repost,
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
            nostr::Kind::WalletConnectInfo => Self::WalletConnectInfo,
            nostr::Kind::Reporting => Self::Reporting,
            nostr::Kind::ZapRequest => Self::ZapRequest,
            #[allow(deprecated)]
            nostr::Kind::ZapReceipt | nostr::Kind::Zap => Self::ZapReceipt,
            nostr::Kind::MuteList => Self::MuteList,
            nostr::Kind::PinList => Self::PinList,
            nostr::Kind::RelayList => Self::RelayList,
            nostr::Kind::Authentication => Self::Authentication,
            nostr::Kind::WalletConnectRequest => Self::WalletConnectRequest,
            nostr::Kind::WalletConnectResponse => Self::WalletConnectResponse,
            nostr::Kind::NostrConnect => Self::NostrConnect,
            nostr::Kind::CategorizedPeopleList => Self::CategorizedPeopleList,
            nostr::Kind::CategorizedBookmarkList => Self::CategorizedBookmarkList,
            nostr::Kind::LiveEvent => Self::LiveEvent,
            nostr::Kind::LiveEventMessage => Self::LiveEventMessage,
            nostr::Kind::ProfileBadges => Self::ProfileBadges,
            nostr::Kind::BadgeDefinition => Self::BadgeDefinition,
            nostr::Kind::LongFormTextNote => Self::LongFormTextNote,
            nostr::Kind::ApplicationSpecificData => Self::ApplicationSpecificData,
            nostr::Kind::FileMetadata => Self::FileMetadataK,
            nostr::Kind::HttpAuth => Self::HttpAuth,
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
            KindEnum::MetadataK => Self::Metadata,
            KindEnum::TextNote => Self::TextNote,
            KindEnum::RecommendRelay => Self::RecommendRelay,
            KindEnum::ContactList => Self::ContactList,
            KindEnum::EncryptedDirectMessage => Self::EncryptedDirectMessage,
            KindEnum::EventDeletion => Self::EventDeletion,
            KindEnum::Repost => Self::Repost,
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
            KindEnum::WalletConnectInfo => Self::WalletConnectInfo,
            KindEnum::Reporting => Self::Reporting,
            KindEnum::ZapRequest => Self::ZapRequest,
            KindEnum::ZapReceipt => Self::ZapReceipt,
            KindEnum::MuteList => Self::MuteList,
            KindEnum::PinList => Self::PinList,
            KindEnum::RelayList => Self::RelayList,
            KindEnum::Authentication => Self::Authentication,
            KindEnum::WalletConnectRequest => Self::WalletConnectRequest,
            KindEnum::WalletConnectResponse => Self::WalletConnectResponse,
            KindEnum::NostrConnect => Self::NostrConnect,
            KindEnum::CategorizedPeopleList => Self::CategorizedPeopleList,
            KindEnum::CategorizedBookmarkList => Self::CategorizedBookmarkList,
            KindEnum::LiveEvent => Self::LiveEvent,
            KindEnum::LiveEventMessage => Self::LiveEventMessage,
            KindEnum::ProfileBadges => Self::ProfileBadges,
            KindEnum::BadgeDefinition => Self::BadgeDefinition,
            KindEnum::LongFormTextNote => Self::LongFormTextNote,
            KindEnum::ApplicationSpecificData => Self::ApplicationSpecificData,
            KindEnum::FileMetadataK => Self::FileMetadata,
            KindEnum::HttpAuth => Self::HttpAuth,
            KindEnum::Regular { kind } => Self::Regular(kind),
            KindEnum::Replaceable { kind } => Self::Replaceable(kind),
            KindEnum::Ephemeral { kind } => Self::Ephemeral(kind),
            KindEnum::ParameterizedReplaceable { kind } => Self::ParameterizedReplaceable(kind),
            KindEnum::Custom { kind } => Self::Custom(kind),
        }
    }
}
