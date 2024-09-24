// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! A local nostr relay

use atomic_destructor::AtomicDestructor;
use nostr_database::prelude::*;

mod internal;
mod session;
mod util;

use self::internal::InternalLocalRelay;
use crate::builder::RelayBuilder;
use crate::error::Error;

/// A local nostr relay
#[derive(Debug, Clone)]
pub struct LocalRelay {
    inner: AtomicDestructor<InternalLocalRelay>,
}

impl LocalRelay {
    /// Run local relay from [`RelayBuilder`]
    #[inline]
    pub async fn run(builder: RelayBuilder) -> Result<Self, Error> {
        Ok(Self {
            inner: AtomicDestructor::new(InternalLocalRelay::run(builder).await?),
        })
    }

    /// Get url
    #[inline]
    pub fn url(&self) -> String {
        self.inner.url()
    }

    /// Get hidden service address if available
    #[inline]
    #[cfg(feature = "tor")]
    pub fn hidden_service(&self) -> Option<&str> {
        self.inner.hidden_service()
    }

    /// Shutdown relay
    #[inline]
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }
}
