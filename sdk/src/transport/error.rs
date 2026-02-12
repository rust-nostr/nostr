//! Transport error

use std::fmt;
use std::io::{self, ErrorKind};

/// Transport Error
#[derive(Debug)]
pub enum TransportError {
    /// I/O error
    IO(io::Error),
    /// An error happened in the underlying backend.
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    Backend(Box<dyn std::error::Error + Send + Sync>),
    /// An error happened in the underlying backend.
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    Backend(Box<dyn std::error::Error>),
}

impl std::error::Error for TransportError {}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => e.fmt(f),
            Self::Backend(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for TransportError {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl TransportError {
    /// Timeout error
    #[inline]
    pub fn timeout() -> Self {
        Self::IO(io::Error::new(ErrorKind::TimedOut, "timeout"))
    }

    /// Create a new backend error
    ///
    /// Shorthand for `Self::Backend(Box::new(error))`.
    #[inline]
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    pub fn backend<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Backend(error.into())
    }

    /// Create a new backend error
    ///
    /// Shorthand for `Self::Backend(Box::new(error))`.
    #[inline]
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    pub fn backend<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error>>,
    {
        Self::Backend(error.into())
    }
}
