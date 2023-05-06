// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(unused_variables)]

use std::str::FromStr;

use log::LevelFilter;

#[cfg(target_os = "android")]
use android_logger::Config;

pub fn init_logger(level: Option<String>) {
    let level: LevelFilter = max_level(level);

    #[cfg(target_os = "android")]
    android_logger::init_once(Config::default().with_max_level(level));
}

fn max_level(level: Option<String>) -> LevelFilter {
    if let Some(level_str) = level {
        if let Ok(level) = LevelFilter::from_str(&level_str) {
            return level;
        }
    }

    if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}
