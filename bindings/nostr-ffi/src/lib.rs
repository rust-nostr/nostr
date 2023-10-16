// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

mod error;
mod event;
pub mod helper;
mod key;
mod message;
mod nips;
mod types;
mod util;

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // External
    //pub use nostr::nips::nip44::Version as NIP44Version;
    pub use nostr::Alphabet;

    // Error
    pub use crate::error::NostrError;

    // Nostr
    pub use crate::event::{
        Event, EventBuilder, EventId, Tag, TagEnum, TagKind, TagKindKnown, UnsignedEvent,
    };
    pub use crate::key::{Keys, PublicKey, SecretKey};
    pub use crate::message::{ClientMessage, Filter, RelayMessage};
    pub use crate::nips::nip04::{nip04_decrypt, nip04_encrypt};
    pub use crate::nips::nip05::{get_nip05_profile, verify_nip05};
    pub use crate::nips::nip11::RelayInformationDocument;
    //pub use crate::nips::nip44::{nip44_decrypt, nip44_encrypt};
    pub use crate::nips::nip46::NostrConnectURI;
    pub use crate::nips::nip57::ZapRequestData;
    pub use crate::nips::nip94::FileMetadata;
    pub use crate::types::{Contact, ImageDimensions, Metadata, Profile, Timestamp};
    pub use crate::util::generate_shared_key;

    pub fn git_hash_version() -> String {
        nostr::git_hash_version().to_string()
    }

    // UDL
    uniffi::include_scaffolding!("nostr");
}
pub use ffi::*;
