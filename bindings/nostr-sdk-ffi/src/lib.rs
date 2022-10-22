// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

// Extenal
use nostr_sdk_base::KindBase;

mod base;
mod error;
mod helper;
mod sdk;

// Error
use self::error::NostrError;

// Base
use self::base::event::{Event, Kind};
use self::base::key::Keys;
use self::base::subscription::SubscriptionFilter;

// SDK
use self::sdk::subscription::{Channel, Subscription};

#[allow(missing_docs)]
mod ffi {
    use super::*;
    uniffi_macros::include_scaffolding!("nostrsdk");
}
pub use ffi::*;
