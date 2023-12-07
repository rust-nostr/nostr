// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{JsonUtil, Url};
use uniffi::Object;

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;

#[derive(Clone, Object)]
pub struct Metadata {
    inner: nostr::Metadata,
}

impl Deref for Metadata {
    type Target = nostr::Metadata;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::Metadata> for Metadata {
    fn from(inner: nostr::Metadata) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Metadata {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::Metadata::new(),
        })
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::Metadata::from_json(json)?,
        }))
    }

    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }

    pub fn set_name(self: Arc<Self>, name: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.name(name);
        Arc::new(builder)
    }

    pub fn get_name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    pub fn set_display_name(self: Arc<Self>, display_name: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.display_name(display_name);
        Arc::new(builder)
    }

    pub fn get_display_name(&self) -> Option<String> {
        self.inner.display_name.clone()
    }

    pub fn set_about(self: Arc<Self>, about: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.about(about);
        Arc::new(builder)
    }

    pub fn get_about(&self) -> Option<String> {
        self.inner.about.clone()
    }

    pub fn set_website(self: Arc<Self>, website: String) -> Result<Arc<Self>> {
        let website = Url::parse(&website)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.website(website);
        Ok(Arc::new(builder))
    }

    pub fn get_website(&self) -> Option<String> {
        self.inner.website.clone()
    }

    pub fn set_picture(self: Arc<Self>, picture: String) -> Result<Arc<Self>> {
        let picture = Url::parse(&picture)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.picture(picture);
        Ok(Arc::new(builder))
    }

    pub fn get_picture(&self) -> Option<String> {
        self.inner.picture.clone()
    }

    pub fn set_banner(self: Arc<Self>, banner: String) -> Result<Arc<Self>> {
        let banner = Url::parse(&banner)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.banner(banner);
        Ok(Arc::new(builder))
    }

    pub fn get_banner(&self) -> Option<String> {
        self.inner.banner.clone()
    }

    pub fn set_nip05(self: Arc<Self>, nip05: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.nip05(nip05);
        Arc::new(builder)
    }

    pub fn get_nip05(&self) -> Option<String> {
        self.inner.nip05.clone()
    }

    pub fn set_lud06(self: Arc<Self>, lud06: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.lud06(lud06);
        Arc::new(builder)
    }

    pub fn get_lud06(&self) -> Option<String> {
        self.inner.lud06.clone()
    }

    pub fn set_lud16(self: Arc<Self>, lud16: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.lud16(lud16);
        Arc::new(builder)
    }

    pub fn get_lud16(&self) -> Option<String> {
        self.inner.lud16.clone()
    }

    pub fn set_custom_field(self: Arc<Self>, key: String, value: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.custom_field(key, value);
        Arc::new(builder)
    }

    pub fn get_custom_field(&self, key: String) -> Option<String> {
        self.inner.custom.get(&key).cloned()
    }
}
