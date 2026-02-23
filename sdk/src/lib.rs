#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`
#![doc = include_str!("../README.md")]

pub mod client;
mod events_tracker;
mod future;
pub mod monitor;
pub mod policy;
mod pool;
pub mod prelude;
pub mod relay;
mod shared;
mod stream;
pub mod transport;
