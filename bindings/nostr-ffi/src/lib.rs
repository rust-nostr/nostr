// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr FFI

#![allow(clippy::new_without_default)]

use uniffi::Object;

mod error;
pub mod event;
pub mod helper;
pub mod key;
pub mod message;
pub mod nips;
pub mod types;
pub mod util;

pub use crate::error::NostrError;
pub use crate::event::{
    Event, EventBuilder, EventId, Kind, KindEnum, RelayMetadata, Tag, TagEnum, TagKind,
    UnsignedEvent,
};
pub use crate::key::{Keys, PublicKey, SecretKey};
pub use crate::message::{ClientMessage, ClientMessageEnum, RelayMessage, RelayMessageEnum};
pub use crate::nips::nip04::{nip04_decrypt, nip04_encrypt};
pub use crate::nips::nip05::{get_nip05_profile, verify_nip05};
pub use crate::nips::nip11::RelayInformationDocument;
pub use crate::nips::nip46::{NostrConnectMessage, NostrConnectMetadata, NostrConnectURI};
pub use crate::nips::nip53::{Image, LiveEvent, LiveEventHost, LiveEventStatus, Person};
pub use crate::nips::nip94::FileMetadata;
pub use crate::types::{
    Alphabet, Contact, Filter, ImageDimensions, Metadata, SingleLetterTag, Timestamp,
};
pub use crate::util::{generate_shared_key, JsonValue};

#[derive(Object)]
pub struct NostrLibrary;

#[uniffi::export]
impl NostrLibrary {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn git_hash_version(&self) -> Option<String> {
        option_env!("GIT_HASH").map(|v| v.to_string())
    }
}

uniffi::setup_scaffolding!("nostr");
