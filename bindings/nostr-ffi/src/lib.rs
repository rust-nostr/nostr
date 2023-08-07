// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub mod error;
pub mod event;
pub mod helper;
pub mod key;
pub mod message;
pub mod nips;
pub mod types;

// Error
pub use self::error::NostrError;

// Nostr
pub use self::event::{
    Event, EventBuilder, EventId, Tag, TagEnum, TagKind, TagKindKnown, UnsignedEvent,
};
pub use self::key::{Keys, PublicKey, SecretKey};
pub use self::message::{ClientMessage, Filter, RelayMessage};
pub use self::nips::nip04::{nip04_decrypt, nip04_encrypt};
pub use self::nips::nip05::{get_nip05_profile, verify_nip05};
pub use self::nips::nip11::RelayInformationDocument;
pub use self::nips::nip46::NostrConnectURI;
pub use self::nips::nip94::FileMetadata;
pub use self::types::{Contact, ImageDimensions, Metadata as AccountMetadata, Profile, Timestamp};

uniffi::include_scaffolding!("nostr");
