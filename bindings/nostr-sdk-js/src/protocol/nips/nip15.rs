// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::JsStringArray;

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
    #[wasm_bindgen(constructor)]
    pub fn new(id: &str, cost: f64) -> Self {
        ShippingMethod::new(id, cost).into()
    }

    #[wasm_bindgen(js_name = getShippingCost)]
    pub fn get_shipping_cost(&self) -> JsShippingCost {
        self.inner.get_shipping_cost().into()
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
    #[wasm_bindgen(constructor)]
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

#[wasm_bindgen(js_name = ProductData)]
pub struct JsProductData {
    inner: ProductData,
}

impl Deref for JsProductData {
    type Target = ProductData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<ProductData> for JsProductData {
    fn from(inner: ProductData) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = ProductData)]
impl JsProductData {
    #[wasm_bindgen(constructor)]
    pub fn new(id: &str, stall_id: &str, name: &str, currency: &str) -> Self {
        ProductData::new(id, stall_id, name, currency).into()
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter, js_name = stallId)]
    pub fn stall_id(&self) -> String {
        self.inner.stall_id.clone()
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
    pub fn images(&self) -> Option<Vec<String>> {
        self.inner.images.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn currency(&self) -> String {
        self.inner.currency.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn price(&self) -> f64 {
        self.inner.price
    }

    #[wasm_bindgen(getter)]
    pub fn quantity(&self) -> f64 {
        self.inner.quantity as f64
    }

    #[wasm_bindgen(getter)]
    pub fn specs(&self) -> Option<Vec<JsStringArray>> {
        self.inner.specs.clone().map(|v| {
            v.into_iter()
                .map(|s| {
                    s.into_iter()
                        .map(JsValue::from)
                        .collect::<Array>()
                        .unchecked_into()
                })
                .collect()
        })
    }

    #[wasm_bindgen(getter)]
    pub fn shipping(&self) -> Vec<JsShippingCost> {
        self.inner
            .shipping
            .clone()
            .into_iter()
            .map(|s| s.into())
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn categories(&self) -> Option<Vec<String>> {
        self.inner.categories.clone()
    }
}
