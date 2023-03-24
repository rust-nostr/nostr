// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Thread

#[cfg(feature = "blocking")]
use std::time::Duration;

use nostr_sdk_net::futures_util::Future;
#[cfg(feature = "blocking")]
use tokio::runtime::{Builder, Runtime};

#[cfg(feature = "blocking")]
fn new_current_thread() -> nostr::Result<Runtime> {
    Ok(Builder::new_current_thread().enable_all().build()?)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Impossible to join thread")]
    JoinError,
}

#[allow(dead_code)]
pub enum JoinHandle<T> {
    Std(std::thread::JoinHandle<T>),
    Tokio(tokio::task::JoinHandle<T>),
}

impl<T> JoinHandle<T> {
    pub async fn join(self) -> Result<T, Error> {
        match self {
            Self::Std(handle) => handle.join().map_err(|_| Error::JoinError),
            Self::Tokio(handle) => handle.await.map_err(|_| Error::JoinError),
        }
    }
}

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
