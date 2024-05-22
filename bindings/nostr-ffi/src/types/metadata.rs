// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::serde_json::Value;
use nostr::{JsonUtil, Url};
use uniffi::{Object, Record};

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::JsonValue;

#[derive(Record)]
pub struct MetadataRecord {
    /// Name
    pub name: Option<String>,
    /// Display name
    pub display_name: Option<String>,
    /// Description
    pub about: Option<String>,
    /// Website url
    pub website: Option<String>,
    /// Picture url
    pub picture: Option<String>,
    /// Banner url
    pub banner: Option<String>,
    /// NIP05 (ex. name@example.com)
    pub nip05: Option<String>,
    /// LNURL
    pub lud06: Option<String>,
    /// Lightning Address
    pub lud16: Option<String>,
}

impl From<MetadataRecord> for nostr::Metadata {
    fn from(value: MetadataRecord) -> Self {
        Self {
            name: value.name,
            display_name: value.display_name,
            about: value.about,
            website: value.website,
            picture: value.picture,
            banner: value.banner,
            nip05: value.nip05,
            lud06: value.lud06,
            lud16: value.lud16,
            ..Default::default()
        }
    }
}

impl From<nostr::Metadata> for MetadataRecord {
    fn from(value: nostr::Metadata) -> Self {
        Self {
            name: value.name,
            display_name: value.display_name,
            about: value.about,
            website: value.website,
            picture: value.picture,
            banner: value.banner,
            nip05: value.nip05,
            lud06: value.lud06,
            lud16: value.lud16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
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
    pub fn new() -> Self {
        Self {
            inner: nostr::Metadata::new(),
        }
    }

    #[uniffi::constructor]
    pub fn from_record(r: MetadataRecord) -> Self {
        Self { inner: r.into() }
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::Metadata::from_json(json)?,
        })
    }

    pub fn as_record(&self) -> MetadataRecord {
        self.inner.clone().into()
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    pub fn set_name(self: Arc<Self>, name: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.name(name);
        builder
    }

    pub fn get_name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    pub fn set_display_name(self: Arc<Self>, display_name: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.display_name(display_name);
        builder
    }

    pub fn get_display_name(&self) -> Option<String> {
        self.inner.display_name.clone()
    }

    pub fn set_about(self: Arc<Self>, about: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.about(about);
        builder
    }

    pub fn get_about(&self) -> Option<String> {
        self.inner.about.clone()
    }

    pub fn set_website(self: Arc<Self>, website: String) -> Result<Self> {
        let website = Url::parse(&website)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.website(website);
        Ok(builder)
    }

    pub fn get_website(&self) -> Option<String> {
        self.inner.website.clone()
    }

    pub fn set_picture(self: Arc<Self>, picture: String) -> Result<Self> {
        let picture = Url::parse(&picture)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.picture(picture);
        Ok(builder)
    }

    pub fn get_picture(&self) -> Option<String> {
        self.inner.picture.clone()
    }

    pub fn set_banner(self: Arc<Self>, banner: String) -> Result<Self> {
        let banner = Url::parse(&banner)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.banner(banner);
        Ok(builder)
    }

    pub fn get_banner(&self) -> Option<String> {
        self.inner.banner.clone()
    }

    pub fn set_nip05(self: Arc<Self>, nip05: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.nip05(nip05);
        builder
    }

    pub fn get_nip05(&self) -> Option<String> {
        self.inner.nip05.clone()
    }

    pub fn set_lud06(self: Arc<Self>, lud06: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.lud06(lud06);
        builder
    }

    pub fn get_lud06(&self) -> Option<String> {
        self.inner.lud06.clone()
    }

    pub fn set_lud16(self: Arc<Self>, lud16: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.lud16(lud16);
        builder
    }

    pub fn get_lud16(&self) -> Option<String> {
        self.inner.lud16.clone()
    }

    pub fn set_custom_field(self: Arc<Self>, key: String, value: JsonValue) -> Result<Self> {
        let value: Value = value.try_into()?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.custom_field(key, value);
        Ok(builder)
    }

    pub fn get_custom_field(&self, key: String) -> Result<Option<JsonValue>> {
        match self.inner.custom.get(&key).cloned() {
            Some(value) => Ok(Some(value.try_into()?)),
            None => Ok(None),
        }
    }
}
