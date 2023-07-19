// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

mod error;
mod event;
pub mod helper;
mod key;
mod message;
mod nips;
mod types;

#[allow(missing_docs)]
#[allow(unused_imports)]
mod ffi {
    // Error
    pub use crate::error::NostrError;

    // Nostr
    pub use crate::event::{Event, EventBuilder, EventId, UnsignedEvent};
    pub use crate::key::{Keys, PublicKey, SecretKey};
    pub use crate::message::{ClientMessage, Filter, RelayMessage};
    pub use crate::nips::nip04::{nip04_decrypt, nip04_encrypt};
    pub use crate::nips::nip11::RelayInformationDocument;
    pub use crate::types::{Contact, Metadata as AccountMetadata, Timestamp};

    // UDL
    uniffi_macros::include_scaffolding!("nostr");
}
pub use ffi::*;
