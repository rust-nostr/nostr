// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt::Debug;
use std::sync::Arc;

use nostr_ffi::{Event, RelayMessage};

#[uniffi::export(callback_interface)]
pub trait HandleNotification: Send + Sync + Debug {
    fn handle_msg(&self, relay_url: String, msg: Arc<RelayMessage>);
    fn handle(&self, relay_url: String, subscription_id: String, event: Arc<Event>);
}
