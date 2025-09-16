// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Blossom error

use std::fmt;

use nostr::event::builder;
use nostr::signer::SignerError;
use nostr::types::ParseError;
use reqwest::Response;
use reqwest::header::{InvalidHeaderValue, ToStrError};

/// Blossom error
#[derive(Debug)]
pub enum Error {
    /// Nostr signer error
    Signer(SignerError),
    /// Event builder error
    EventBuilder(builder::Error),
    /// Reqwest error
    Reqwest(reqwest::Error),
    /// Invalid header value
    InvalidHeaderValue(InvalidHeaderValue),
    /// Url parse error
    Url(ParseError),
    /// To string error
    ToStr(ToStrError),
    /// Response error
    Response {
        /// Prefix for the error message
        prefix: String,
        /// Response
        res: Response,
    },
    /// Returned when a redirect URL does not contain the expected hash
    RedirectUrlDoesNotContainSha256,
    /// Returned when a redirect response is missing the Location header
    RedirectResponseMissingLocationHeader,
}

impl Error {
    #[inline]
    pub(super) fn response<S>(prefix: S, res: Response) -> Self
    where
        S: Into<String>,
    {
        Self::Response {
            prefix: prefix.into(),
            res,
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signer(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::Reqwest(e) => write!(f, "{e}"),
            Self::InvalidHeaderValue(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
            Self::ToStr(e) => write!(f, "{e}"),
            Self::Response { prefix, res } => {
                let reason: &str = res
                    .headers()
                    .get("X-Reason")
                    .map(|h| h.to_str().unwrap_or("Unknown reason"))
                    .unwrap_or_else(|| "No reason provided");
                write!(f, "{prefix}: {} - {reason}", res.status())
            }
            Self::RedirectUrlDoesNotContainSha256 => {
                write!(f, "Redirect URL does not contain SHA256")
            }
            Self::RedirectResponseMissingLocationHeader => {
                write!(f, "Redirect response missing 'Location' header")
            }
        }
    }
}

impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e)
    }
}

impl From<builder::Error> for Error {
    fn from(e: builder::Error) -> Self {
        Self::EventBuilder(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(e: InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<ToStrError> for Error {
    fn from(e: ToStrError) -> Self {
        Self::ToStr(e)
    }
}
