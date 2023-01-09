// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

mod client;
mod error;
mod logger;
mod subscription;
mod thread;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> error::Result<Self>;
}

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // Extenal
    pub use nostr::util::time::timestamp;
    pub use nostr_ffi::{AccountMetadata, Contact, Event, EventBuilder, Keys, SubscriptionFilter};

    // Namespace
    pub use crate::logger::init_logger;

    // Nostr SDK
    pub use crate::client::{Client, HandleNotification};
    pub use crate::error::NostrSdkError;
    pub use crate::subscription::{Channel, Subscription};

    // UDL
    uniffi_macros::include_scaffolding!("nostrsdk");
}
pub use ffi::*;
