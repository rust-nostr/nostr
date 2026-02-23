// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! A mock relay for (unit) tests.

use std::ops::Deref;

use nostr::prelude::*;
use nostr_database::prelude::*;

use crate::builder::{LocalRelayBuilder, LocalRelayTestOptions};
use crate::error::Error;
use crate::local::LocalRelay;

/// A mock relay for (unit) tests.
///
/// Check [`LocalRelay`] for more details.
#[derive(Debug, Clone)]
pub struct MockRelay {
    local: LocalRelay,
}

impl Deref for MockRelay {
    type Target = LocalRelay;

    fn deref(&self) -> &Self::Target {
        &self.local
    }
}

impl MockRelay {
    async fn new(builder: LocalRelayBuilder) -> Result<Self, Error> {
        let relay: LocalRelay = builder.build()?;
        relay.run().await?;
        Ok(Self { local: relay })
    }

    /// Run mock relay
    #[inline]
    pub async fn run() -> Result<Self, Error> {
        let builder = LocalRelayBuilder::default();
        Self::new(builder).await
    }

    /// Run unresponsive relay
    #[inline]
    pub async fn run_with_opts(opts: LocalRelayTestOptions) -> Result<Self, Error> {
        let builder = LocalRelayBuilder::default().test(opts);
        Self::new(builder).await
    }
}
