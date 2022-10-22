// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use log::Level;

#[cfg(target_os = "android")]
use android_logger::{Config, FilterBuilder};

pub fn init_logger() {
    #[allow(unused_variables)]
    let min_level = if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Info
    };

    #[cfg(target_os = "android")]
    android_logger::init_once(Config::default().with_min_level(min_level));
}
