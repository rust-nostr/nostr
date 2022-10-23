// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(unused_variables)]
#![allow(dead_code)]

use std::str::FromStr;

use log::Level;

#[cfg(target_os = "android")]
use android_logger::Config;

pub fn init_logger(level: Option<String>) {
    #[cfg(target_os = "android")]
    android_logger::init_once(Config::default().with_min_level(min_level(level)));
}

fn min_level(level: Option<String>) -> Level {
    if let Some(level_str) = level {
        if let Ok(level) = Level::from_str(&level_str) {
            return level;
        }
    }

    if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Info
    }
}
