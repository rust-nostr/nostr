// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;

mod client;
mod logger;
mod subscription;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> Result<Self>;
}

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // Extenal
    pub use nostr::util::time::timestamp;
    pub use nostr_ffi::{
        AccountMetadata, Contact, Event, EventBuilder, Keys, Kind, KindBase, SubscriptionFilter,
    };

    // Namespace
    pub use crate::logger::init_logger;

    // Nostr SDK
    pub use crate::client::{Client, HandleNotification};
    pub use crate::subscription::{Channel, Subscription};

    // UDL
    uniffi_macros::include_scaffolding!("nostrsdk");
}
pub use ffi::*;
