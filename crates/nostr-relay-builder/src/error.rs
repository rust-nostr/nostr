// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay builder error

use nostr::Event;
use nostr_sdk::client;
use tokio::sync::broadcast;

opaquerr::define_kind! {
    /// Relay builder error kind
    pub ErrorKind {
        /// Nostr protocol error
        Protocol => "nostr protocol error",
        /// Database error
        Database => "database error",
        /// I/O error
        IO => "I/O error",
        /// Anything not covered by the stable categories above.
        Other => "other error",
    }
}

opaquerr::define_error! {
    /// Relay builder error
    pub Error(ErrorKind)

    from {
        nostr::error::Error => ErrorKind::Protocol,
        nostr_database::error::Error => ErrorKind::Database,
        std::io::Error => ErrorKind::IO,
        client::Error => ErrorKind::Other,
        async_wsocket::Error => ErrorKind::Other,
        negentropy::Error => ErrorKind::Other,
        tokio::sync::TryAcquireError => ErrorKind::Other,
        broadcast::error::SendError<Event> => ErrorKind::Other,
        faster_hex::Error => ErrorKind::Other,
    }
}
