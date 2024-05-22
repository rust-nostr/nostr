// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip15;
use nostr::JsonUtil;
use uniffi::{Object, Record};

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;

/// Payload for creating or updating stall
#[derive(Record)]
pub struct StallDataRecord {
    /// UUID of the stall generated by merchant
    pub id: String,
    /// Stall name
    pub name: String,
    /// Stall description
    pub description: Option<String>,
    /// Currency used
    pub currency: String,
    /// Available shipping methods
    pub shipping: Vec<ShippingMethodRecord>,
}

impl From<StallDataRecord> for nip15::StallData {
    fn from(value: StallDataRecord) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            currency: value.currency,
            shipping: value.shipping.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<nip15::StallData> for StallDataRecord {
    fn from(value: nip15::StallData) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            currency: value.currency,
            shipping: value.shipping.into_iter().map(|s| s.into()).collect(),
        }
    }
}

#[derive(Object)]
pub struct StallData {
    inner: nip15::StallData,
}

impl Deref for StallData {
    type Target = nip15::StallData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nip15::StallData> for StallData {
    fn from(inner: nip15::StallData) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl StallData {
    #[uniffi::constructor]
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        currency: String,
        shipping: Vec<Arc<ShippingMethod>>,
    ) -> Self {
        Self {
            inner: nip15::StallData {
                id,
                name,
                description,
                currency,
                shipping: shipping
                    .into_iter()
                    .map(|s| s.as_ref().deref().clone())
                    .collect(),
            },
        }
    }

    #[uniffi::constructor]
    pub fn from_record(r: StallDataRecord) -> Self {
        Self { inner: r.into() }
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(nip15::StallData::from_json(json)?.into())
    }

    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    pub fn currency(&self) -> String {
        self.inner.currency.clone()
    }

    pub fn shipping(&self) -> Vec<Arc<ShippingMethod>> {
        self.inner
            .shipping
            .iter()
            .cloned()
            .map(|s| Arc::new(s.into()))
            .collect()
    }

    pub fn as_record(&self) -> StallDataRecord {
        self.inner.clone().into()
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }
}

/// Payload for creating or updating product
#[derive(Record)]
pub struct ProductData {
    /// UUID of the product generated by merchant
    pub id: String,
    /// Id of the stall that this product belongs to
    pub stall_id: String,
    /// Product name
    pub name: String,
    /// Description of the product
    pub description: Option<String>,
    /// Image urls of the product
    pub images: Option<Vec<String>>,
    /// Currency used
    pub currency: String,
    /// Price of the product
    pub price: f64,
    /// Available items
    pub quantity: u64,
    /// Specifications of the product
    pub specs: Option<Vec<Vec<String>>>,
    /// Shipping method costs
    pub shipping: Vec<ShippingCost>,
    /// Categories of the product (will be added to tags)
    pub categories: Option<Vec<String>>,
}

impl From<ProductData> for nip15::ProductData {
    fn from(value: ProductData) -> Self {
        Self {
            id: value.id,
            stall_id: value.stall_id,
            name: value.name,
            description: value.description,
            images: value.images,
            currency: value.currency,
            price: value.price,
            quantity: value.quantity,
            specs: value.specs,
            shipping: value.shipping.into_iter().map(|s| s.into()).collect(),
            categories: value.categories,
        }
    }
}

#[derive(Record)]
pub struct ShippingMethodRecord {
    /// Shipping method unique id by merchant
    pub id: String,
    /// Shipping method name
    pub name: Option<String>,
    /// Shipping method cost (currency is the same as the stall)
    pub cost: f64,
    /// Covered regions
    pub regions: Vec<String>,
}

impl From<nip15::ShippingMethod> for ShippingMethodRecord {
    fn from(value: nip15::ShippingMethod) -> Self {
        Self {
            id: value.id,
            name: value.name,
            cost: value.cost,
            regions: value.regions,
        }
    }
}

impl From<ShippingMethodRecord> for nip15::ShippingMethod {
    fn from(value: ShippingMethodRecord) -> Self {
        Self {
            id: value.id,
            name: value.name,
            cost: value.cost,
            regions: value.regions,
        }
    }
}

#[derive(Clone, Object)]
pub struct ShippingMethod {
    inner: nip15::ShippingMethod,
}

impl Deref for ShippingMethod {
    type Target = nip15::ShippingMethod;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nip15::ShippingMethod> for ShippingMethod {
    fn from(value: nip15::ShippingMethod) -> Self {
        Self { inner: value }
    }
}

#[uniffi::export]
impl ShippingMethod {
    /// Create a new shipping method
    #[uniffi::constructor]
    pub fn new(id: String, cost: f64) -> Self {
        Self {
            inner: nip15::ShippingMethod::new(id, cost),
        }
    }

    /// Set the name of the shipping method
    pub fn name(self: Arc<Self>, name: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.name(name);
        builder
    }

    /// Add a region to the shipping method
    pub fn regions(self: Arc<Self>, regions: Vec<String>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.regions(regions);
        builder
    }

    /// Get the product shipping cost of the shipping method
    pub fn get_shipping_cost(&self) -> ShippingCost {
        self.inner.get_shipping_cost().into()
    }
}

/// Delivery cost for shipping method as defined by the merchant in the product
#[derive(Record)]
pub struct ShippingCost {
    /// Id of the shipping method
    pub id: String,
    /// Cost to use this shipping method
    pub cost: f64,
}

impl From<ShippingCost> for nip15::ShippingCost {
    fn from(value: ShippingCost) -> Self {
        Self {
            id: value.id,
            cost: value.cost,
        }
    }
}

impl From<nip15::ShippingCost> for ShippingCost {
    fn from(value: nip15::ShippingCost) -> Self {
        Self {
            id: value.id,
            cost: value.cost,
        }
    }
}
