// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip15::{ShippingCost, ShippingMethod, StallData};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = ShippingCost)]
pub struct JsShippingCost {
    inner: ShippingCost,
}

impl From<ShippingCost> for JsShippingCost {
    fn from(inner: ShippingCost) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = ShippingCost)]
impl JsShippingCost {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn cost(&self) -> f64 {
        self.inner.cost
    }
}

#[wasm_bindgen(js_name = ShippingMethod)]
pub struct JsShippingMethod {
    inner: ShippingMethod,
}

impl From<ShippingMethod> for JsShippingMethod {
    fn from(inner: ShippingMethod) -> Self {
        Self { inner }
    }
}

impl Deref for JsShippingMethod {
    type Target = ShippingMethod;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = ShippingMethod)]
impl JsShippingMethod {
    pub fn new(id: &str, cost: f64) -> Self {
        ShippingMethod::new(id, cost).into()
    }

    pub fn get_shipping_cost(&self) -> JsShippingCost {
        ShippingMethod::get_shipping_cost(self.deref()).into()
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn cost(&self) -> f64 {
        self.inner.cost
    }

    #[wasm_bindgen(getter)]
    pub fn regions(&self) -> Vec<String> {
        self.inner.regions.clone()
    }
}

#[wasm_bindgen(js_name = StallData)]
pub struct JsStallData {
    inner: StallData,
}

impl From<StallData> for JsStallData {
    fn from(inner: StallData) -> Self {
        Self { inner }
    }
}

impl Deref for JsStallData {
    type Target = StallData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = StallData)]
impl JsStallData {
    pub fn new(id: &str, name: &str, currency: &str) -> Self {
        StallData::new(id, name, currency).into()
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn currency(&self) -> String {
        self.inner.currency.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn shipping(&self) -> Vec<JsShippingMethod> {
        self.inner
            .shipping
            .clone()
            .into_iter()
            .map(|s| s.into())
            .collect()
    }
}
