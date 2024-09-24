// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Builder

use std::net::IpAddr;
use std::sync::Arc;

use nostr_database::prelude::*;

/// Rate limit
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Max active REQs
    pub max_reqs: usize,
    /// Max events per minutes
    pub notes_per_minute: u32,
    //pub whitelist: Option<Vec<String>>,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_reqs: 20,
            notes_per_minute: 60,
        }
    }
}

/// Relay builder
pub struct RelayBuilder {
    /// IP address
    pub addr: Option<IpAddr>,
    /// Port
    pub port: Option<u16>,
    /// Database
    pub database: Arc<DynNostrDatabase>,
    /// Rate limit
    pub rate_limit: RateLimit,
}

impl Default for RelayBuilder {
    fn default() -> Self {
        Self {
            addr: None,
            port: None,
            database: Arc::new(MemoryDatabase::with_opts(MemoryDatabaseOptions {
                events: true,
                max_events: Some(75_000),
            })),
            rate_limit: RateLimit::default(),
        }
    }
}

impl RelayBuilder {
    /// Set IP address
    #[inline]
    pub fn addr(mut self, ip: IpAddr) -> Self {
        self.addr = Some(ip);
        self
    }

    /// Set port
    #[inline]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set database
    #[inline]
    pub fn database<D>(mut self, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        self.database = database.into_nostr_database();
        self
    }

    /// Set rate limit
    #[inline]
    pub fn rate_limit(mut self, limit: RateLimit) -> Self {
        self.rate_limit = limit;
        self
    }
}
