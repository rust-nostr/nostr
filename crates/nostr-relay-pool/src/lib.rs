// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Relay Pool

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

pub mod pool;
pub mod prelude;
pub mod relay;
mod util;

pub use self::pool::options::RelayPoolOptions;
pub use self::pool::{Output, RelayPool, RelayPoolNotification};
pub use self::relay::flags::{AtomicRelayServiceFlags, RelayServiceFlags};
pub use self::relay::limits::RelayLimits;
pub use self::relay::options::{
    FilterOptions, NegentropyDirection, NegentropyOptions, RelayOptions, RelaySendOptions,
    SubscribeAutoCloseOptions, SubscribeOptions,
};
pub use self::relay::stats::RelayConnectionStats;
pub use self::relay::{Reconciliation, Relay, RelayBlacklist, RelayNotification, RelayStatus};
