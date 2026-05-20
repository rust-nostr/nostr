//! Nostr Keyring error.

use std::{error, fmt};

#[cfg(feature = "async")]
use async_utility::tokio;

opaquerr::define_kind! {
    /// Category for a [`Error`].
    pub ErrorKind {
        /// Nostr protocol error.
        Protocol => "nostr protocol error",
        /// Keyring error.
        Keyring => "keyring error",
        /// Anything not covered by the stable categories above.
        Other => "other error",
    }
}

enum Inner {
    Protocol(nostr::error::Error),
    Keyring(keyring::Error),
    #[cfg(feature = "async")]
    Join(tokio::task::JoinError),
}

impl Inner {
    const fn kind(&self) -> ErrorKind {
        match self {
            Inner::Protocol(_) => ErrorKind::Protocol,
            Inner::Keyring(_) => ErrorKind::Keyring,
            #[cfg(feature = "async")]
            Inner::Join(_) => ErrorKind::Other,
        }
    }
}

/// Nostr keyring error
pub struct Error(Inner);

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind();

        match &self.0 {
            Inner::Protocol(e) => f.debug_tuple("Error").field(&kind).field(e).finish(),
            Inner::Keyring(e) => f.debug_tuple("Error").field(&kind).field(e).finish(),
            #[cfg(feature = "async")]
            Inner::Join(e) => f.debug_tuple("Error").field(&kind).field(e).finish(),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.0 {
            Inner::Protocol(e) => Some(e),
            Inner::Keyring(e) => Some(e),
            #[cfg(feature = "async")]
            Inner::Join(e) => Some(e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Inner::Protocol(e) => e.fmt(f),
            Inner::Keyring(e) => e.fmt(f),
            #[cfg(feature = "async")]
            Inner::Join(e) => e.fmt(f),
        }
    }
}

impl Error {
    /// Returns the error category.
    #[inline]
    pub const fn kind(&self) -> ErrorKind {
        self.0.kind()
    }
}

impl From<nostr::error::Error> for Error {
    #[inline]
    fn from(inner: nostr::error::Error) -> Self {
        Self(Inner::Protocol(inner))
    }
}

impl From<keyring::Error> for Error {
    #[inline]
    fn from(inner: keyring::Error) -> Self {
        Self(Inner::Keyring(inner))
    }
}

#[cfg(feature = "async")]
impl From<tokio::task::JoinError> for Error {
    #[inline]
    fn from(inner: tokio::task::JoinError) -> Self {
        Self(Inner::Join(inner))
    }
}
