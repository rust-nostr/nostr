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
///
/// This is automatically shutdown when all instances/clones are dropped!
#[derive(Debug, Clone)]
pub struct LocalRelay {
    inner: AtomicDestructor<InnerLocalRelay>,
}

impl LocalRelay {
    /// Create a new local relay
    #[inline]
    pub fn new(builder: RelayBuilder) -> Self {
        Self {
            inner: AtomicDestructor::new(InnerLocalRelay::new(builder)),
        }
    }

    /// Run the local relay
    #[inline]
    pub async fn run(&self) -> Result<(), Error> {
        self.inner.run().await?;
        Ok(())
    }

    /// Get url
    #[inline]
    pub async fn url(&self) -> RelayUrl {
        self.inner.url().await
    }

    /// Run and get the hidden service address
    #[inline]
    #[cfg(feature = "tor")]
    pub async fn hidden_service(&self) -> Result<Option<&str>, Error> {
        let addr: &Option<String> = self.inner.hidden_service().await?;
        Ok(addr.as_deref())
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
