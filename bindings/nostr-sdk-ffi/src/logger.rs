// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use tracing::Level;
#[cfg(any(target_os = "android", target_os = "ios"))]
use tracing_subscriber::filter::Targets;
#[cfg(any(target_os = "android", target_os = "ios"))]
use tracing_subscriber::fmt;
#[cfg(any(target_os = "android", target_os = "ios"))]
use tracing_subscriber::layer::SubscriberExt;
#[cfg(any(target_os = "android", target_os = "ios"))]
use tracing_subscriber::util::SubscriberInitExt;
#[cfg(any(target_os = "android", target_os = "ios"))]
use tracing_subscriber::Layer;
use uniffi::Enum;

#[derive(Enum)]
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

#[uniffi::export]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn init_logger(level: LogLevel) {
    let level: Level = level.into();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            tracing::info!("Desktop logger initialized");

            // Log git hash (defined at compile time)
            match crate::git_hash_version() {
                Some(hash) => tracing::info!("Git hash: {hash}"),
                None => tracing::warn!("Git hash not defined!"),
            };
        }
        Err(e) => eprintln!("Impossible to init desktop logger: {e}"),
    }
}

#[uniffi::export]
#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn init_logger(level: LogLevel) {
    let level: Level = level.into();

    #[cfg(target_os = "android")]
    let layer = fmt::layer().with_writer(paranoid_android::AndroidLogMakeWriter::new(
        "rust.nostr.sdk".to_owned(),
    ));

    #[cfg(not(target_os = "android"))]
    let layer = fmt::layer();

    let targets = Targets::new().with_default(level);

    let res = tracing_subscriber::registry()
        .with(layer.with_ansi(false).with_file(false).with_filter(targets))
        .try_init();

    match res {
        Ok(_) => {
            tracing::info!("Mobile logger initialized");

            // Log git hash (defined at compile time)
            match crate::git_hash_version() {
                Some(hash) => tracing::info!("Git hash: {hash}"),
                None => tracing::warn!("Git hash not defined!"),
            };
        }
        Err(e) => eprintln!("Impossible to init mobile logger: {e}"),
    }
}
