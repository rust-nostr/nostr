// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::thread;
use std::time::Duration;

use anyhow::Result;

pub fn spawn<F>(name: &'static str, f: F) -> thread::JoinHandle<()>
where
    F: 'static + Send + FnOnce() -> Result<()>,
{
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || {
            if let Err(e) = f() {
                log::warn!("{} thread failed: {}", name, e);
            }
        })
        .expect("failed to spawn a thread")
}

pub fn sleep(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds));
}

pub fn sleep_millis(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

pub fn panicking() -> bool {
    thread::panicking()
}
