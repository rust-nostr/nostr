// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::future::Future;

use js_sys::Promise;
use wasm_bindgen::{JsError, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;

use crate::error::Result;

pub(crate) fn future_to_promise<F, T>(future: F) -> Promise
where
    F: Future<Output = Result<T>> + 'static,
    T: Into<JsValue>,
{
    let mut future = Some(future);
    Promise::new(&mut |resolve, reject| {
        let future = future.take().unwrap_throw();
        spawn_local(async move {
            match future.await {
                Ok(value) => resolve
                    .call1(&JsValue::UNDEFINED, &value.into())
                    .unwrap_throw(),
                Err(_) => reject
                    .call1(
                        &JsValue::UNDEFINED,
                        &JsError::new("Impossible to execute promise").into(),
                    )
                    .unwrap_throw(),
            };
        });
    })
}
