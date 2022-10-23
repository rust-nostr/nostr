// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;

mod base;
mod error;
mod helper;
mod logger;
mod sdk;
mod thread;
mod util;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> Result<Self>;
}

#[allow(missing_docs)]
mod ffi {
    // Extenal
    pub use nostr_sdk_base::event::KindBase;

    // Namespace
    pub use crate::logger::init_logger;
    pub use crate::util::time::timestamp;

    // Error
    pub use crate::error::NostrError;

    // Base
    pub use crate::base::event::{Contact, Event, Kind};
    pub use crate::base::key::Keys;
    pub use crate::base::subscription::SubscriptionFilter;

    // SDK
    pub use crate::sdk::client::Client;
    pub use crate::sdk::relay::RelayPoolNotifications;
    pub use crate::sdk::subscription::{Channel, Subscription};

    uniffi_macros::include_scaffolding!("nostrsdk");
}
pub use ffi::*;
