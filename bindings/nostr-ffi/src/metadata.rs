// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use nostr::Metadata as MetadataSdk;
use url::Url;

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

    pub fn name(self: Arc<Self>, name: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.name(name);

        Arc::new(builder)
    }

    pub fn display_name(self: Arc<Self>, display_name: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.display_name(display_name);

        Arc::new(builder)
    }

    pub fn about(self: Arc<Self>, about: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.about(about);

        Arc::new(builder)
    }

    pub fn picture(self: Arc<Self>, picture: String) -> Result<Arc<Self>> {
        let picture = Url::from_str(&picture)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.picture(picture);

        Ok(Arc::new(builder))
    }

    pub fn nip05(self: Arc<Self>, nip05: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.metadata = builder.metadata.nip05(nip05);

        Arc::new(builder)
    }
}
