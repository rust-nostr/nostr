// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Urls

use alloc::string::{String, ToString};
use core::convert::Infallible;
use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "std")]
pub use url::*;
#[cfg(not(feature = "std"))]
pub use url_fork::*;

/// Relay URL error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Url parse error
    Url(ParseError),
    /// Unsupported URL scheme
    UnsupportedScheme(String),
    /// Multiple scheme separators
    MultipleSchemeSeparators,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(e) => write!(f, "{e}"),
            Self::UnsupportedScheme(scheme) => write!(f, "Unsupported scheme: {scheme}"),
            Self::MultipleSchemeSeparators => write!(f, "Multiple scheme separators"),
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

/// Relay URL
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelayUrl {
    url: Url,
    has_trailing_slash: bool,
}

impl fmt::Debug for RelayUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let url: &str = self.as_str();
        f.debug_tuple("RelayUrl").field(&url).finish()
    }
}

impl RelayUrl {
    /// Parse relay URL
    #[inline]
    pub fn parse<S>(url: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let url: &str = url.as_ref();

        // Check that "://" appears only once in the URL
        if url.matches("://").count() > 1 {
            return Err(Error::MultipleSchemeSeparators);
        }

        // Check if has trailing slash
        let has_trailing_slash: bool = url.ends_with('/');

        // Parse URL
        let url: Url = Url::parse(url)?;

        // Check scheme
        match url.scheme() {
            "ws" | "wss" => Ok(Self {
                url,
                has_trailing_slash,
            }),
            scheme => Err(Error::UnsupportedScheme(scheme.to_string())),
        }
    }

    /// Check if the URL is a hidden onion service address
    #[inline]
    pub fn is_onion(&self) -> bool {
        self.url
            .domain()
            .map_or(false, |host| host.ends_with(".onion"))
    }

    /// Return the serialization of this relay URL without the trailing slash.
    ///
    /// This method will always remove the trailing slash.
    #[inline]
    pub fn as_str_without_trailing_slash(&self) -> &str {
        self.url.as_str().trim_end_matches('/')
    }

    /// Return the serialization of this relay URL.
    ///
    /// The trailing slash will be removed only if the parsed URL hadn't it.
    #[inline]
    pub fn as_str(&self) -> &str {
        if !self.has_trailing_slash {
            return self.as_str_without_trailing_slash();
        }

        self.url.as_str()
    }
}

impl fmt::Display for RelayUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RelayUrl {
    type Err = Error;

    fn from_str(relay_url: &str) -> Result<Self, Self::Err> {
        Self::parse(relay_url)
    }
}

impl Serialize for RelayUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RelayUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url: String = String::deserialize(deserializer)?;
        Self::parse(url).map_err(serde::de::Error::custom)
    }
}

impl From<RelayUrl> for Url {
    fn from(relay_url: RelayUrl) -> Self {
        relay_url.url
    }
}

impl<'a> From<&'a RelayUrl> for &'a Url {
    fn from(relay_url: &'a RelayUrl) -> Self {
        &relay_url.url
    }
}

/// Try into relay URL
pub trait TryIntoUrl {
    /// Error
    type Err: fmt::Debug;

    /// Try into relay URL
    fn try_into_url(self) -> Result<RelayUrl, Self::Err>;
}

impl TryIntoUrl for RelayUrl {
    type Err = Infallible;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        Ok(self)
    }
}

impl TryIntoUrl for &RelayUrl {
    type Err = Infallible;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        Ok(self.clone())
    }
}

impl TryIntoUrl for Url {
    type Err = Error;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        RelayUrl::parse(self.as_str())
    }
}

impl TryIntoUrl for &Url {
    type Err = Error;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        RelayUrl::parse(self.as_str())
    }
}

impl TryIntoUrl for String {
    type Err = Error;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        RelayUrl::parse(self)
    }
}

impl TryIntoUrl for &String {
    type Err = Error;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        RelayUrl::parse(self)
    }
}

impl TryIntoUrl for &str {
    type Err = Error;

    #[inline]
    fn try_into_url(self) -> Result<RelayUrl, Self::Err> {
        RelayUrl::parse(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_url_valid() {
        assert!(RelayUrl::parse("ws://127.0.0.1:7777").is_ok());
        assert!(RelayUrl::parse("wss://relay.damus.io").is_ok());
        assert!(RelayUrl::parse("ws://example.com").is_ok());
        assert!(RelayUrl::parse("wss://example.com/path/to/resource").is_ok());
    }

    #[test]
    fn test_relay_url_invalid() {
        assert_eq!(
            RelayUrl::parse("https://relay.damus.io").unwrap_err(),
            Error::UnsupportedScheme(String::from("https"))
        );
        assert_eq!(
            RelayUrl::parse("ftp://relay.damus.io").unwrap_err(),
            Error::UnsupportedScheme(String::from("ftp"))
        );
        assert_eq!(
            RelayUrl::parse("wss://relay.damus.io,ws://127.0.0.1:7777").unwrap_err(),
            Error::MultipleSchemeSeparators
        );
        assert_eq!(
            RelayUrl::parse("wss://relay.damus.iowss://127.0.0.1:8888").unwrap_err(),
            Error::MultipleSchemeSeparators
        );
        assert_eq!(
            RelayUrl::parse("wss://").unwrap_err(),
            Error::Url(ParseError::EmptyHost)
        );
    }

    #[test]
    fn test_relay_url_as_str() {
        let relay_url = RelayUrl::parse("ws://example.com").unwrap();
        assert_eq!(relay_url.as_str(), "ws://example.com");

        let relay_url = RelayUrl::parse("ws://example.com/").unwrap();
        assert_eq!(relay_url.as_str(), "ws://example.com/");

        let relay_url = RelayUrl::parse("ws://example.com/").unwrap();
        assert_eq!(
            relay_url.as_str_without_trailing_slash(),
            "ws://example.com"
        );
    }

    #[test]
    fn test_relay_url_from_str() {
        let relay_url: Result<RelayUrl, _> = "ws://example.com".parse();
        assert!(relay_url.is_ok());
    }

    #[test]
    fn test_serde_relay_url() {
        let relay_url = RelayUrl::parse("ws://example.com").unwrap();
        let serialized = serde_json::to_string(&relay_url).unwrap();
        let deserialized: RelayUrl = serde_json::from_str(&serialized).unwrap();
        assert_eq!(relay_url, deserialized);
    }

    #[test]
    fn test_is_onion() {
        // Onion
        let onion_url =
            RelayUrl::parse("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
                .unwrap();
        assert!(onion_url.is_onion());

        // Non onion
        let non_onion_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        assert!(!non_onion_url.is_onion());
        let non_onion_url = RelayUrl::parse("ws://example.com:81").unwrap();
        assert!(!non_onion_url.is_onion());
        let non_onion_url = RelayUrl::parse("ws://127.0.0.1:7777").unwrap();
        assert!(!non_onion_url.is_onion());
    }
}
