// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip05: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            name: None,
            display_name: None,
            about: None,
            picture: None,
            nip05: None,
        }
    }

    /// Set name
    pub fn name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set display_name
    pub fn display_name(self, display_name: impl Into<String>) -> Self {
        Self {
            display_name: Some(display_name.into()),
            ..self
        }
    }

    /// Set about
    pub fn about(self, about: impl Into<String>) -> Self {
        Self {
            about: Some(about.into()),
            ..self
        }
    }

    /// Set name
    pub fn picture(self, picture: Url) -> Self {
        Self {
            picture: Some(picture.to_string()),
            ..self
        }
    }

    /// Set nip05
    pub fn nip05(self, nip05: impl Into<String>) -> Self {
        Self {
            nip05: Some(nip05.into()),
            ..self
        }
    }
}
