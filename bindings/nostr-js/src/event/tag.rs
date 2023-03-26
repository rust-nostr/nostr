// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Tags, typescript_type = "Array<Array<string>>")]
    pub type JsTags;
}
