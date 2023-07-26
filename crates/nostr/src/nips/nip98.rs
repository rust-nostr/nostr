use url::Url;

use crate::Tag;

/// HTTP Data
pub struct HttpData {
    /// absolute request URL
    pub url: Url,
    /// HTTP method
    pub method: String,
    /// SHA256 hash of the request body
    pub payload: Option<String>,
}

impl HttpData {
    /// New [`HttpData`]
    pub fn new<S>(url: Url, method: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url,
            method: method.into(),
            payload: None,
        }
    }

    /// Add hex-encoded SHA256 hash of the request body
    pub fn payload<S>(self, payload: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            payload: Some(payload.into()),
            ..self
        }
    }
}

impl From<HttpData> for Vec<Tag> {
    fn from(value: HttpData) -> Self {
        let mut tags = Vec::new();

        let HttpData {
            url,
            method,
            payload,
        } = value;

        tags.push(Tag::HttpAuthUrl(url));
        tags.push(Tag::Method(method));

        if let Some(payload) = payload {
            tags.push(Tag::Payload(payload))
        }

        tags
    }
}
