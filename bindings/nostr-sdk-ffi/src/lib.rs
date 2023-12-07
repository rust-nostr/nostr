// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

pub use nostr_ffi::{
    generate_shared_key, get_nip05_profile, nip04_decrypt, nip04_encrypt, verify_nip05, Alphabet,
    ClientMessage, Contact, Event, EventBuilder, EventId, FileMetadata, Filter, ImageDimensions,
    Keys, Metadata, NostrConnectURI, NostrError, NostrLibrary, Profile, PublicKey,
    RelayInformationDocument, RelayMessage, SecretKey, Tag, TagEnum, TagKind, TagKindKnown,
    Timestamp, UnsignedEvent, ZapRequestData,
};

mod client;
mod database;
mod error;
mod logger;
mod relay;
mod thread;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> error::Result<Self>;
}

pub use crate::client::{Client, ClientBuilder, HandleNotification, Options};
pub use crate::database::NostrDatabase;
pub use crate::error::NostrSdkError;
pub use crate::logger::{init_logger, LogLevel};
pub use crate::relay::{ActiveSubscription, Relay, RelayConnectionStats, RelayStatus};

uniffi::setup_scaffolding!("nostr_sdk");
