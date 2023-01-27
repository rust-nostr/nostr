// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

mod contact;
mod error;
mod event;
pub mod helper;
mod key;
mod metadata;
mod subscription;

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // Error
    pub use crate::error::NostrError;

    // Nostr
    pub use crate::contact::Contact;
    pub use crate::event::builder::EventBuilder;
    pub use crate::event::Event;
    pub use crate::key::Keys;
    pub use crate::metadata::Metadata as AccountMetadata;
    pub use crate::subscription::SubscriptionFilter;

    // UDL
    uniffi_macros::include_scaffolding!("nostr");
}
pub use ffi::*;
