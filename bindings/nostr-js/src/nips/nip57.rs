// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip57::ZapType;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = ZapType)]
pub enum JsZapType {
    /// Public
    Public,
    /// Private
    Private,
    /// Anonymous
    Anonymous,
}

impl From<JsZapType> for ZapType {
    fn from(value: JsZapType) -> Self {
        match value {
            JsZapType::Public => Self::Public,
            JsZapType::Private => Self::Private,
            JsZapType::Anonymous => Self::Anonymous,
        }
    }
}

impl From<ZapType> for JsZapType {
    fn from(value: ZapType) -> Self {
        match value {
            ZapType::Public => Self::Public,
            ZapType::Private => Self::Private,
            ZapType::Anonymous => Self::Anonymous,
        }
    }
}
