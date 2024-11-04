// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use tracing::Level;

use super::get_git_hash;

#[repr(C)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}

#[no_mangle]
pub extern "C" fn init_logger(level: LogLevel) {
    let level: Level = level.into();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            tracing::info!("Desktop logger initialized");

            // Log git hash (defined at compile time)
            tracing::info!("Git hash: {}", get_git_hash());
        }
        Err(e) => eprintln!("Impossible to init desktop logger: {e}"),
    }
}
