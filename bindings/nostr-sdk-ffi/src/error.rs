// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use uniffi::Error;

pub type Result<T, E = NostrSdkError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[uniffi(flat_error)]
pub enum NostrSdkError {
    Generic(String),
}

impl fmt::Display for NostrSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(e) => write!(f, "{e}"),
        }
    }
}

impl<T> From<T> for NostrSdkError
where
    T: std::error::Error,
{
    fn from(e: T) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

#[derive(Debug)]
pub(crate) struct MiddleError(String);

impl std::error::Error for MiddleError {}

impl fmt::Display for MiddleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl MiddleError {
    pub fn new<S>(msg: S) -> Self
    where
        S: Into<String>,
    {
        Self(msg.into())
    }
}

impl From<NostrSdkError> for MiddleError {
    fn from(e: NostrSdkError) -> Self {
        Self(e.to_string())
    }
}
