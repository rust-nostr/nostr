// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

mod base;
mod error;
mod helper;
mod sdk;

#[allow(missing_docs)]
mod ffi {
    // Extenal
    pub use nostr_sdk_base::event::KindBase;

    // Error
    pub use crate::error::NostrError;

    // Base
    pub use crate::base::event::{Contact, Event, Kind};
    pub use crate::base::key::Keys;
    pub use crate::base::subscription::SubscriptionFilter;

    // SDK
    pub use crate::sdk::subscription::{Channel, Subscription};

    uniffi_macros::include_scaffolding!("nostrsdk");
}
pub use ffi::*;
