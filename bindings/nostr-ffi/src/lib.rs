// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

mod error;
mod event;
pub mod helper;
mod key;
mod message;
mod nips;
mod types;

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // External
    pub use nostr::Alphabet;

    // Error
    pub use crate::error::NostrError;

    // Nostr
    pub use crate::event::{
        Event, EventBuilder, EventId, Tag, TagEnum, TagKind, TagKindKnown, UnsignedEvent,
    };
    pub use crate::key::{Keys, PublicKey, SecretKey};
    pub use crate::message::{ClientMessage, Filter, RelayMessage};
    pub use crate::nips::nip04::{generate_shared_key, nip04_decrypt, nip04_encrypt};
    pub use crate::nips::nip05::{get_nip05_profile, verify_nip05};
    pub use crate::nips::nip11::RelayInformationDocument;
    pub use crate::nips::nip46::NostrConnectURI;
    pub use crate::nips::nip57::ZapRequestData;
    pub use crate::nips::nip94::FileMetadata;
    pub use crate::types::{
        Contact, ImageDimensions, Metadata as AccountMetadata, Profile, Timestamp,
    };

    // UDL
    uniffi::include_scaffolding!("nostr");
}
pub use ffi::*;
