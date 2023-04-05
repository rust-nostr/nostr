// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::any::Any;
use std::fmt;
use std::thread::Result;

use nostr_sdk_net::futures_util::Future;
use tokio::sync::oneshot::{self, Receiver};
use wasm_bindgen_futures::spawn_local;

pub struct JoinHandle<T>(Receiver<T>);

impl<T> JoinHandle<T> {
    pub async fn join(self) -> Result<T> {
        let res = self.0.await;
        res.map_err(|e| Box::new(e) as Box<(dyn Any + Send + 'static)>)
    }
}

impl<T> fmt::Debug for JoinHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("JoinHandle { .. }")
    }
}

pub fn spawn<T>(f: T) -> JoinHandle<T::Output>
where
    T: Future + 'static,
    T::Output: 'static,
{
    let (sender, receiver) = oneshot::channel();

    spawn_local(async {
        let res = f.await;
        sender.send(res).ok();
    });

    JoinHandle(receiver)
}
