// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr SDK Pool

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)]
#![allow(clippy::arc_with_non_send_sync)]

mod flags;
pub mod limits;
pub mod options;
pub mod pool;
pub mod relay;
mod stats;

pub use self::flags::{AtomicRelayServiceFlags, RelayServiceFlags};
pub use self::limits::Limits;
pub use self::options::{
    FilterOptions, NegentropyDirection, NegentropyOptions, RelayOptions, RelayPoolOptions,
    RelaySendOptions,
};
pub use self::pool::RelayPoolNotification;
pub use self::relay::{ActiveSubscription, InternalSubscriptionId, Relay, RelayStatus};
pub use self::stats::RelayConnectionStats;
