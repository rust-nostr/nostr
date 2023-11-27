// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

mod client;
mod error;
mod logger;
mod relay;
mod thread;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> error::Result<Self>;
}

// External
pub use nostr_ffi::{
    generate_shared_key, get_nip05_profile, git_hash_version, nip04_decrypt, nip04_encrypt,
    verify_nip05, ClientMessage, Contact, Event, EventBuilder, EventId, FileMetadata, Filter,
    ImageDimensions, Keys, Metadata, NostrConnectURI, NostrError, Profile, PublicKey,
    RelayInformationDocument, RelayMessage, SecretKey, Tag, TagEnum, TagKind, TagKindKnown,
    Timestamp, UnsignedEvent, ZapRequestData,
};
pub use nostr_sdk::{Alphabet, RelayStatus};

// Namespace
pub use crate::logger::{init_logger, LogLevel};

// Nostr SDK
pub use crate::client::{Client, ClientBuilder, HandleNotification, Options};
pub use crate::error::NostrSdkError;
pub use crate::relay::{ActiveSubscription, Relay, RelayConnectionStats};

// UDL
uniffi::include_scaffolding!("nostr_sdk");
