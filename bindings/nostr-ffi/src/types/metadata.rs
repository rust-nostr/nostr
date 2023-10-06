// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{JsonUtil, Metadata as MetadataSdk, Url};

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;

#[derive(Clone)]
pub struct Metadata {
    metadata: MetadataSdk,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Metadata {
    type Target = MetadataSdk;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl From<MetadataSdk> for Metadata {
    fn from(metadata: MetadataSdk) -> Self {
        Self { metadata }
    }
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            metadata: MetadataSdk::new(),
        }
    }

    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            metadata: MetadataSdk::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> String {
        self.metadata.as_json()
    }

    pub fn set_name(self: Arc<Self>, name: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.name(name);
        Arc::new(builder)
    }

    pub fn get_name(&self) -> Option<String> {
        self.metadata.name.clone()
    }

    pub fn set_display_name(self: Arc<Self>, display_name: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.display_name(display_name);
        Arc::new(builder)
    }

    pub fn get_display_name(&self) -> Option<String> {
        self.metadata.display_name.clone()
    }

    pub fn set_about(self: Arc<Self>, about: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.about(about);
        Arc::new(builder)
    }

    pub fn get_about(&self) -> Option<String> {
        self.metadata.about.clone()
    }

    pub fn set_website(self: Arc<Self>, website: String) -> Result<Arc<Self>> {
        let website = Url::parse(&website)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.website(website);
        Ok(Arc::new(builder))
    }

    pub fn get_website(&self) -> Option<String> {
        self.metadata.website.clone()
    }

    pub fn set_picture(self: Arc<Self>, picture: String) -> Result<Arc<Self>> {
        let picture = Url::parse(&picture)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.picture(picture);
        Ok(Arc::new(builder))
    }

    pub fn get_picture(&self) -> Option<String> {
        self.metadata.picture.clone()
    }

    pub fn set_banner(self: Arc<Self>, banner: String) -> Result<Arc<Self>> {
        let banner = Url::parse(&banner)?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.banner(banner);
        Ok(Arc::new(builder))
    }

    pub fn get_banner(&self) -> Option<String> {
        self.metadata.banner.clone()
    }

    pub fn set_nip05(self: Arc<Self>, nip05: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.nip05(nip05);
        Arc::new(builder)
    }

    pub fn get_nip05(&self) -> Option<String> {
        self.metadata.nip05.clone()
    }

    pub fn set_lud06(self: Arc<Self>, lud06: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.lud06(lud06);
        Arc::new(builder)
    }

    pub fn get_lud06(&self) -> Option<String> {
        self.metadata.lud06.clone()
    }

    pub fn set_lud16(self: Arc<Self>, lud16: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.lud16(lud16);
        Arc::new(builder)
    }

    pub fn get_lud16(&self) -> Option<String> {
        self.metadata.lud16.clone()
    }
}
