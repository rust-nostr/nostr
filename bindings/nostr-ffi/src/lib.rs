// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

mod contact;
mod error;
mod event;
pub mod helper;
mod key;
mod subscription;

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // Extenral
    pub use nostr::KindBase;

    // Error
    pub use crate::error::NostrError;

    // Nostr
    pub use crate::contact::Contact;
    pub use crate::event::kind::Kind;
    pub use crate::event::Event;
    pub use crate::key::Keys;
    pub use crate::subscription::SubscriptionFilter;

    // UDL
    uniffi_macros::include_scaffolding!("nostr");
}
pub use ffi::*;
