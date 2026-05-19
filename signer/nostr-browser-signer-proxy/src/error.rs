// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Error

use std::{fmt, io};

use hyper::http;
use tokio::sync::oneshot::error::RecvError;

/// Error
#[derive(Debug)]
pub enum Error {
    /// Nostr protocol error
    Protocol(nostr::error::Error),
    /// I/O error
    Io(io::Error),
    /// HTTP error
    Http(http::Error),
    /// Json error
    Json(serde_json::Error),
    /// Oneshot channel receive error
    OneShotRecv(RecvError),
    /// Generic error
    Generic(String),
    /// Timeout
    Timeout,
    /// The server is shutdown
    Shutdown,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Http(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::OneShotRecv(e) => write!(f, "{e}"),
            Self::Generic(e) => write!(f, "{e}"),
            Self::Timeout => write!(f, "timeout"),
            Self::Shutdown => write!(f, "server is shutdown"),
        }
    }
}

impl From<nostr::error::Error> for Error {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Self::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<RecvError> for Error {
    fn from(e: RecvError) -> Self {
        Self::OneShotRecv(e)
    }
}
