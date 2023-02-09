// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;

#[napi(js_name = "Metadata")]
pub struct JsMetadata {
    metadata: Metadata,
}

impl Deref for JsMetadata {
    type Target = Metadata;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl From<Metadata> for JsMetadata {
    fn from(metadata: Metadata) -> Self {
        Self { metadata }
    }
}

#[napi]
impl JsMetadata {
    #[allow(clippy::new_without_default)]
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            metadata: Metadata::new(),
        }
    }

    #[napi(factory)]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            metadata: Metadata::from_json(json).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn as_json(&self) -> Result<String> {
        self.metadata.as_json().map_err(into_err)
    }

    #[napi]
    pub fn name(&self, name: String) -> Self {
        Self {
            metadata: self.metadata.to_owned().name(name),
        }
    }

    #[napi]
    pub fn display_name(&self, display_name: String) -> Self {
        Self {
            metadata: self.metadata.to_owned().display_name(display_name),
        }
    }

    #[napi]
    pub fn about(&self, about: String) -> Self {
        Self {
            metadata: self.metadata.to_owned().about(about),
        }
    }

    #[napi]
    pub fn website(&self, url: String) -> Result<Self> {
        let url = Url::parse(&url).map_err(into_err)?;
        Ok(Self {
            metadata: self.metadata.to_owned().website(url),
        })
    }

    #[napi]
    pub fn picture(&self, url: String) -> Result<Self> {
        let url = Url::parse(&url).map_err(into_err)?;
        Ok(Self {
            metadata: self.metadata.to_owned().picture(url),
        })
    }

    #[napi]
    pub fn banner(&self, url: String) -> Result<Self> {
        let url = Url::parse(&url).map_err(into_err)?;
        Ok(Self {
            metadata: self.metadata.to_owned().banner(url),
        })
    }

    #[napi]
    pub fn nip05(&self, nip05: String) -> Self {
        Self {
            metadata: self.metadata.to_owned().nip05(nip05),
        }
    }

    #[napi]
    pub fn lud06(&self, lud06: String) -> Self {
        Self {
            metadata: self.metadata.to_owned().lud06(lud06),
        }
    }

    #[napi]
    pub fn lud16(&self, lud16: String) -> Self {
        Self {
            metadata: self.metadata.to_owned().lud16(lud16),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_metadata() {
        let content = r#"{"name":"myname","about":"Description","display_name":""}"#;
        let metadata = Metadata::from_json(content).unwrap();
        assert_eq!(
            metadata,
            Metadata::new()
                .name("myname")
                .about("Description")
                .display_name("")
        );
    }
}
