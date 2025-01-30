// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::{JsEvent, JsUnsignedEvent};
use crate::protocol::key::JsPublicKey;
use crate::signer::JsNostrSigner;

/// Unwrapped Gift Wrap
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[wasm_bindgen(js_name = UnwrappedGift)]
pub struct JsUnwrappedGift {
    inner: UnwrappedGift,
}

impl From<UnwrappedGift> for JsUnwrappedGift {
    fn from(inner: UnwrappedGift) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = UnwrappedGift)]
impl JsUnwrappedGift {
    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    #[wasm_bindgen(js_name = fromGiftWrap)]
    pub async fn from_gift_wrap(
        signer: &JsNostrSigner,
        gift_wrap: &JsEvent,
    ) -> Result<JsUnwrappedGift> {
        Ok(Self {
            inner: UnwrappedGift::from_gift_wrap(signer.deref(), gift_wrap.deref())
                .await
                .map_err(into_err)?,
        })
    }

    /// Get sender public key
    #[wasm_bindgen(getter)]
    pub fn sender(&self) -> JsPublicKey {
        self.inner.sender.into()
    }

    /// Get rumor
    #[wasm_bindgen(getter)]
    pub fn rumor(&self) -> JsUnsignedEvent {
        self.inner.rumor.clone().into()
    }
}
