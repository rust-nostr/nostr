// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Metadata

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use core::fmt;

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url_fork::Url;

/// [`Metadata`] error
#[derive(Debug)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "Json: {e}"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Metadata
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    /// Name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub name: Option<String>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub display_name: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub about: Option<String>,
    /// Website url
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub website: Option<String>,
    /// Picture url
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub picture: Option<String>,
    /// Banner url
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub banner: Option<String>,
    /// NIP05 (ex. name@example.com)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub nip05: Option<String>,
    /// LNURL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub lud06: Option<String>,
    /// Lightning Address
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub lud16: Option<String>,
    /// Custom fields
    #[serde(
        flatten,
        serialize_with = "serialize_custom_fields",
        deserialize_with = "deserialize_custom_fields"
    )]
    #[serde(default)]
    pub custom: BTreeMap<String, Value>,
}

impl Metadata {
    /// New empty [`Metadata`]
    pub fn new() -> Self {
        Self::default()
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

    /// Set custom metadata field
    pub fn custom_field<S>(self, field_name: S, value: Value) -> Self
    where
        S: Into<String>,
    {
        let mut custom: BTreeMap<String, Value> = self.custom;
        custom.insert(field_name.into(), value);
        Self { custom, ..self }
    }
}

fn serialize_custom_fields<S>(
    custom_fields: &BTreeMap<String, Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(custom_fields.len()))?;
    for (field_name, value) in custom_fields {
        map.serialize_entry(field_name, value)?;
    }
    map.end()
}

fn deserialize_custom_fields<'de, D>(deserializer: D) -> Result<BTreeMap<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = BTreeMap<String, Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("TODO")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut custom_fields = BTreeMap::new();
            while let Some(field_name) = map.next_key::<String>()? {
                let value: Value = map.next_value()?;
                custom_fields.insert(field_name, value);
            }
            Ok(custom_fields)
        }
    }

    deserializer.deserialize_map(GenericTagsVisitor)
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
        assert_eq!(
            metadata,
            Metadata::new()
                .name("myname")
                .about("Description")
                .custom_field("displayName", Value::String("Jack".into()))
        );

        let content = r#"{"lud16":"thesimplekid@cln.thesimplekid.com","nip05":"_@thesimplekid.com","display_name":"thesimplekid","about":"Wannabe open source dev","name":"thesimplekid","username":"thesimplekid","displayName":"thesimplekid","lud06":""}"#;
        let metadata = Metadata::from_json(content).unwrap();
        assert_eq!(
            metadata,
            Metadata::new()
                .name("thesimplekid")
                .display_name("thesimplekid")
                .about("Wannabe open source dev")
                .nip05("_@thesimplekid.com")
                .lud06("")
                .lud16("thesimplekid@cln.thesimplekid.com")
                .custom_field("username", Value::String("thesimplekid".into()))
                .custom_field("displayName", Value::String("thesimplekid".into()))
        )
    }
}
