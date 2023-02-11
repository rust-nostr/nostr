// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate napi_derive;

pub use nostr_nodejs;

mod client;
mod error;
mod relay;

pub use self::client::{JsClient, JsOptions};
pub use self::relay::{JsRelay, JsRelayStatus};

#[napi]
pub fn init_logger() {
    env_logger::init();
}
