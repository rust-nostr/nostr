// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Thread

use std::time::Duration;

use nostr_sdk_net::futures_util::Future;
#[cfg(feature = "blocking")]
use tokio::runtime::{Builder, Runtime};

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(feature = "blocking")]
fn new_current_thread() -> nostr::Result<Runtime> {
    Ok(Builder::new_current_thread().enable_all().build()?)
}

/// Thread Error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Join Error
    #[error("Impossible to join thread")]
    JoinError,
}

/// Join Handle
pub enum JoinHandle<T> {
    /// Std
    #[cfg(not(target_arch = "wasm32"))]
    Std(std::thread::JoinHandle<T>),
    /// Tokio
    #[cfg(not(target_arch = "wasm32"))]
    Tokio(tokio::task::JoinHandle<T>),
    /// Wasm
    #[cfg(target_arch = "wasm32")]
    Wasm(self::wasm::JoinHandle<T>),
}

impl<T> JoinHandle<T> {
    /// Join
    pub async fn join(self) -> Result<T, Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Std(handle) => handle.join().map_err(|_| Error::JoinError),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(handle) => handle.await.map_err(|_| Error::JoinError),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(handle) => handle.join().await.map_err(|_| Error::JoinError),
        }
    }
}

/// Spawn
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<T>(future: T) -> Option<JoinHandle<T::Output>>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    #[cfg(feature = "blocking")]
    match new_current_thread() {
        Ok(rt) => {
            let handle = std::thread::spawn(move || {
                let res = rt.block_on(future);
                rt.shutdown_timeout(Duration::from_millis(100));
                res
            });
            Some(JoinHandle::Std(handle))
        }
        Err(e) => {
            log::error!("Impossible to create new thread: {:?}", e);
            None
        }
    }

    #[cfg(not(feature = "blocking"))]
    {
        let handle = tokio::task::spawn(future);
        Some(JoinHandle::Tokio(handle))
    }
}

/// Spawn
#[cfg(target_arch = "wasm32")]
pub fn spawn<T>(future: T) -> Option<JoinHandle<T::Output>>
where
    T: Future + 'static,
{
    let handle = self::wasm::spawn(future);
    Some(JoinHandle::Wasm(handle))
}

/// Sleep
pub async fn sleep(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(duration).await;
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::sleep(duration).await;
}
