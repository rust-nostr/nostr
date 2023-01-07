// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#[cfg(feature = "blocking")]
use std::time::Duration;

use futures_util::Future;
#[cfg(feature = "blocking")]
use tokio::runtime::{Builder, Runtime};

#[cfg(feature = "blocking")]
fn new_current_thread() -> nostr::Result<Runtime> {
    Ok(Builder::new_current_thread().enable_all().build()?)
}

pub fn spawn<T>(future: T)
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    #[cfg(feature = "blocking")]
    match new_current_thread() {
        Ok(rt) => {
            std::thread::spawn(move || {
                rt.block_on(future);
                rt.shutdown_timeout(Duration::from_millis(100));
            });
        }
        Err(e) => log::error!("Impossible to create new thread: {:?}", e),
    };

    #[cfg(not(feature = "blocking"))]
    tokio::task::spawn(future);
}
