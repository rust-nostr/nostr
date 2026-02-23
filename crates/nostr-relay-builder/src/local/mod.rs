// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! A local nostr relay

use std::net::SocketAddr;

use atomic_destructor::AtomicDestructor;
use nostr_sdk::client::SyncSummary;
use nostr_sdk::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};

mod inner;
mod session;
mod util;

use self::inner::InnerLocalRelay;
use crate::builder::LocalRelayBuilder;
use crate::error::Error;

/// A local nostr relay
///
/// This is automatically shutdown when all instances/clones are dropped!
#[derive(Debug, Clone)]
pub struct LocalRelay {
    inner: AtomicDestructor<InnerLocalRelay>,
}

impl LocalRelay {
    /// Create a new local relay with the default configuration.
    ///
    /// Use [`LocalRelay::builder`] for customizing it!
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    /// Create a new local relay builder
    #[inline]
    pub fn builder() -> LocalRelayBuilder {
        LocalRelayBuilder::default()
    }

    #[inline]
    pub(super) fn from_builder(builder: LocalRelayBuilder) -> Result<Self, Error> {
        Ok(Self {
            inner: AtomicDestructor::new(InnerLocalRelay::new(builder)?),
        })
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

    /// Sync events with other relay(s).
    #[inline]
    pub async fn sync_with<'a, I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: SyncOptions,
    ) -> Result<Output<SyncSummary>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        self.inner.sync_with(urls, filter, opts).await
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

    /// Save the event to the database and, if success, notify the subscribers.
    pub async fn add_event(&self, event: Event) -> Result<SaveEventStatus, Error> {
        let status = self.inner.save_event(&event).await?;

        if status.is_success() {
            self.inner.notify_event(event);
        }

        Ok(status)
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
