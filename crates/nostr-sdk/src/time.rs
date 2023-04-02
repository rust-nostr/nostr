// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Time module

use std::future::Future;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use nostr_sdk_net::futures_util::future::{AbortHandle, Abortable};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub(crate) async fn timeout<F>(timeout: Option<Duration>, future: F) -> Option<F::Output>
where
    F: Future,
{
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(timeout) = timeout {
        tokio::time::timeout(timeout, future).await.ok()
    } else {
        Some(future.await)
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(timeout) = timeout {
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            let future = Abortable::new(future, abort_registration);
            spawn_local(async move {
                gloo_timers::callback::Timeout::new(timeout.as_millis() as u32, move || {
                    abort_handle.abort();
                })
                .forget();
            });
            future.await.ok()
        } else {
            Some(future.await)
        }
    }
}
