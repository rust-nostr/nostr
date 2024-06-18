// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_relay_pool::prelude::*;

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientNotification {
    /// Private direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[cfg(feature = "nip59")]
    PrivateDirectMessage {
        // TODO: convert to GiftWrap so it's more generic?
        /// The sender of the private message
        sender: PublicKey,
        /// The message
        message: String,
        /// The timestamp of the message (NOT the tweaked one!)
        timestamp: Timestamp,
        /// Event ID
        reply_to: Option<EventId>,
        // TODO: other?
    },
    /// Relay Pool notification
    Pool(RelayPoolNotification),
}
