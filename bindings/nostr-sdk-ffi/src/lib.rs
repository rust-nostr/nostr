// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

use nostr_sdk::Timestamp;

mod client;
mod error;
mod logger;
mod relay;
mod thread;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> error::Result<Self>;
}

pub fn timestamp() -> u64 {
    Timestamp::now().as_u64()
}

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // External
    pub use nostr_ffi::{
        nip04_decrypt, nip04_encrypt, AccountMetadata, ClientMessage, Contact, Event, EventBuilder,
        Filter, Keys, NostrError, PublicKey, RelayMessage, SecretKey,
    };
    pub use nostr_sdk::RelayStatus;

    // Namespace
    pub use crate::logger::init_logger;
    pub use crate::timestamp;

    // Nostr SDK
    pub use crate::client::{Client, HandleNotification, Options};
    pub use crate::error::NostrSdkError;
    pub use crate::relay::{ActiveSubscription, Relay, RelayConnectionStats};

    // UDL
    uniffi_macros::include_scaffolding!("nostr_sdk");
}
pub use ffi::*;
