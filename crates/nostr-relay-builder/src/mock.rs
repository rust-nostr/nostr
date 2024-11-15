// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! A mock relay for (unit) tests.

use std::ops::Deref;

use nostr::prelude::*;
use nostr_database::prelude::*;

use crate::builder::{RelayBuilder, RelayTestOptions};
use crate::error::Error;
use crate::local::LocalRelay;

/// A mock relay for (unit) tests.
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
    /// Run mock relay
    #[inline]
    pub async fn run() -> Result<Self, Error> {
        let builder = RelayBuilder::default();
        Ok(Self {
            local: LocalRelay::run(builder).await?,
        })
    }

    /// Run unresponsive relay
    #[inline]
    pub async fn run_with_opts(opts: RelayTestOptions) -> Result<Self, Error> {
        let builder = RelayBuilder::default().test(opts);
        Ok(Self {
            local: LocalRelay::run(builder).await?,
        })
    }
}
