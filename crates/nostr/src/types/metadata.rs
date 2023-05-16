// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Metadata

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
#[cfg(all(feature = "alloc", not(feature = "std")))]
use core::error::Error as StdError;
use core::fmt;
#[cfg(feature = "std")]
use std::error::Error as StdError;

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use url::Url;

/// [`Metadata`] error
#[derive(Debug)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "json error: {e}"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
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
    /// Custom fields
    pub custom: Map<String, Value>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    /// New empty [`Metadata`]
    pub fn new() -> Self {
        Self {
            name: None,
            display_name: None,
            about: None,
            website: None,
            picture: None,
            banner: None,
            nip05: None,
            lud06: None,
            lud16: None,
            custom: Map::new(),
        }
    }

    /// Deserialize [`Metadata`] from `JSON` string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(serde_json::from_str(&json.into())?)
    }

    /// Serialize [`Metadata`] to `JSON` string
    pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
    }

    /// Set name
    pub fn name<S>(self, name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set display_name
    pub fn display_name<S>(self, display_name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            display_name: Some(display_name.into()),
            ..self
        }
    }

    /// Set about
    pub fn about<S>(self, about: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            about: Some(about.into()),
            ..self
        }
    }

    /// Set website
    pub fn website(self, url: Url) -> Self {
        Self {
            website: Some(url.into()),
            ..self
        }
    }

    /// Set picture
    pub fn picture(self, url: Url) -> Self {
        Self {
            picture: Some(url.into()),
            ..self
        }
    }

    /// Set banner
    pub fn banner(self, url: Url) -> Self {
        Self {
            banner: Some(url.into()),
            ..self
        }
    }

    /// Set nip05
    pub fn nip05<S>(self, nip05: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            nip05: Some(nip05.into()),
            ..self
        }
    }

    /// Set lud06 (LNURL)
    pub fn lud06<S>(self, lud06: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            lud06: Some(lud06.into()),
            ..self
        }
    }

    /// Set lud16 (Lightning Address)
    pub fn lud16<S>(self, lud16: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            lud16: Some(lud16.into()),
            ..self
        }
    }

    /// Set custom metadata
    pub fn custom(self, map: Map<String, Value>) -> Self {
        Self {
            custom: map,
            ..self
        }
    }
}

impl Serialize for Metadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len: usize = 9 + self.custom.len();
        let mut map = serializer.serialize_map(Some(len))?;
        if let Some(value) = &self.name {
            map.serialize_entry("name", &json!(value))?;
        }
        if let Some(value) = &self.display_name {
            map.serialize_entry("display_name", &json!(value))?;
        }
        if let Some(value) = &self.about {
            map.serialize_entry("about", &json!(value))?;
        }
        if let Some(value) = &self.website {
            map.serialize_entry("website", &json!(value))?;
        }
        if let Some(value) = &self.picture {
            map.serialize_entry("picture", &json!(value))?;
        }
        if let Some(value) = &self.banner {
            map.serialize_entry("banner", &json!(value))?;
        }
        if let Some(value) = &self.nip05 {
            map.serialize_entry("nip05", &json!(value))?;
        }
        if let Some(value) = &self.lud06 {
            map.serialize_entry("lud06", &json!(value))?;
        }
        if let Some(value) = &self.lud16 {
            map.serialize_entry("lud16", &json!(value))?;
        }
        for (k, v) in &self.custom {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MetadataVisitor)
    }
}

struct MetadataVisitor;

impl<'de> Visitor<'de> for MetadataVisitor {
    type Value = Metadata;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Metadata, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut f: Metadata = Metadata::new();

        if let Some(Value::String(name)) = map.remove("name") {
            f.name = Some(name);
        }

        if let Some(Value::String(display_name)) = map.remove("display_name") {
            f.display_name = Some(display_name);
        }

        if let Some(Value::String(about)) = map.remove("about") {
            f.about = Some(about);
        }

        if let Some(Value::String(website)) = map.remove("website") {
            f.website = Some(website);
        }

        if let Some(Value::String(picture)) = map.remove("picture") {
            f.picture = Some(picture);
        }

        if let Some(Value::String(banner)) = map.remove("banner") {
            f.banner = Some(banner);
        }

        if let Some(Value::String(nip05)) = map.remove("nip05") {
            f.nip05 = Some(nip05);
        }

        if let Some(Value::String(lud06)) = map.remove("lud06") {
            f.lud06 = Some(lud06);
        }

        if let Some(Value::String(lud16)) = map.remove("lud16") {
            f.lud16 = Some(lud16);
        }

        f.custom = map;

        Ok(f)
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

        let content = r#"{"name":"myname","about":"Description","displayName":"Jack"}"#;
        let metadata = Metadata::from_json(content).unwrap();
        let mut custom = Map::new();
        custom.insert("displayName".into(), Value::String("Jack".into()));
        assert_eq!(
            metadata,
            Metadata::new()
                .name("myname")
                .about("Description")
                .custom(custom)
        );

        let content = r#"{"lud16":"thesimplekid@cln.thesimplekid.com","nip05":"_@thesimplekid.com","display_name":"thesimplekid","about":"Wannabe open source dev","name":"thesimplekid","username":"thesimplekid","displayName":"thesimplekid","lud06":""}"#;
        let metadata = Metadata::from_json(content).unwrap();
        let mut custom = Map::new();
        custom.insert("username".into(), Value::String("thesimplekid".into()));
        custom.insert("displayName".into(), Value::String("thesimplekid".into()));
        assert_eq!(
            metadata,
            Metadata::new()
                .name("thesimplekid")
                .display_name("thesimplekid")
                .about("Wannabe open source dev")
                .nip05("_@thesimplekid.com")
                .lud06("")
                .lud16("thesimplekid@cln.thesimplekid.com")
                .custom(custom)
        )
    }
}
