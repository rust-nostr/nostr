// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! A local nostr relay

use std::net::SocketAddr;

use atomic_destructor::AtomicDestructor;
use nostr_database::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};

mod inner;
mod session;
mod util;

use self::inner::InnerLocalRelay;
use crate::builder::RelayBuilder;
use crate::error::Error;

/// A local nostr relay
#[derive(Debug, Clone)]
pub struct LocalRelay {
    inner: AtomicDestructor<InnerLocalRelay>,
}

impl LocalRelay {
    /// Create a new local relay without listening
    #[inline]
    pub async fn new(builder: RelayBuilder) -> Result<Self, Error> {
        Ok(Self {
            inner: AtomicDestructor::new(InnerLocalRelay::new(builder).await?),
        })
    }

    /// Run local relay from [`RelayBuilder`]
    #[inline]
    pub async fn run(builder: RelayBuilder) -> Result<Self, Error> {
        Ok(Self {
            inner: AtomicDestructor::new(InnerLocalRelay::run(builder).await?),
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

    /// Send event to subscribers
    ///
    /// Return `true` if the event is successfully sent.
    ///
    /// This method doesn't save the event into the database!
    /// It's intended to be used ONLY when the database is shared with other apps (i.e. with the nostr-sdk `Client`).
    pub fn notify_event(&self, event: Event) -> bool {
        self.inner.notify_event(event)
    }

    /// Shutdown relay
    #[inline]
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }

    /// Pass an already upgraded stream
    pub async fn take_connection<S>(&self, stream: S, addr: SocketAddr) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        self.inner.handle_upgraded_connection(stream, addr).await
    }
}
