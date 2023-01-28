// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::Timestamp;

mod client;
mod error;
mod logger;
mod subscription;
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
    // Extenal
    pub use nostr_ffi::{AccountMetadata, Contact, Event, EventBuilder, Keys, SubscriptionFilter};

    // Namespace
    pub use crate::logger::init_logger;
    pub use crate::timestamp;

    // Nostr SDK
    pub use crate::client::{Client, HandleNotification};
    pub use crate::error::NostrSdkError;
    pub use crate::subscription::{Channel, Subscription};

    // UDL
    uniffi_macros::include_scaffolding!("nostrsdk");
}
pub use ffi::*;
