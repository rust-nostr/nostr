#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub use async_utility;
#[doc(hidden)]
pub use async_wsocket::ConnectionMode;
#[doc(hidden)]
pub use nostr::{self, *};

pub mod client;
pub mod monitor;
pub mod policy;
pub mod pool;
pub mod prelude;
pub mod relay;
mod shared;
pub mod stream;
pub mod transport;

pub use self::client::{Client, ClientBuilder, ClientOptions};
pub use self::pool::options::RelayPoolOptions;
pub use self::pool::{Output, RelayPool, RelayPoolNotification};
pub use self::relay::capabilities::{AtomicRelayCapabilities, RelayCapabilities};
pub use self::relay::limits::RelayLimits;
pub use self::relay::options::{
    RelayOptions, SubscribeAutoCloseOptions, SubscribeOptions, SyncDirection, SyncOptions,
};
pub use self::relay::stats::RelayConnectionStats;
pub use self::relay::{Reconciliation, Relay, RelayNotification, RelayStatus};
