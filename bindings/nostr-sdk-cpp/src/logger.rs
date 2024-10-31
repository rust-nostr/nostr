// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use tracing::Level;

use super::git_hash_version;

#[cxx::bridge(namespace = "logger")]
mod ffi {
    enum LogLevel {
        Error,
        Warn,
        Info,
        Debug,
        Trace,
    }

    extern "Rust" {
        fn init(level: LogLevel);
    }
}

impl From<ffi::LogLevel> for Level {
    fn from(value: ffi::LogLevel) -> Self {
        match value {
            ffi::LogLevel::Trace => Self::TRACE,
            ffi::LogLevel::Debug => Self::DEBUG,
            ffi::LogLevel::Info => Self::INFO,
            ffi::LogLevel::Warn => Self::WARN,
            ffi::LogLevel::Error => Self::ERROR,
            _ => unreachable!(),
        }
    }
}

pub fn init(level: ffi::LogLevel) {
    let level: Level = level.into();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            tracing::info!("Desktop logger initialized");

            // Log git hash (defined at compile time)
            tracing::info!("Git hash: {}", git_hash_version())
        }
        Err(e) => eprintln!("Impossible to init desktop logger: {e}"),
    }
}
