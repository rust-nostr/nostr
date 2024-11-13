// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Builder

use std::net::{IpAddr, SocketAddr};
#[cfg(all(feature = "tor", any(target_os = "android", target_os = "ios")))]
use std::path::Path;
#[cfg(feature = "tor")]
use std::path::PathBuf;
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

/// Relay builder tor hidden service options
#[cfg(feature = "tor")]
pub struct RelayBuilderHiddenService {
    /// Nickname (local identifier) for a Tor hidden service
    ///
    /// Used to look up this service's keys, state, configuration, etc., and distinguish them from other services.
    pub nickname: String,
    /// Custom path
    pub custom_path: Option<PathBuf>,
}

#[cfg(feature = "tor")]
impl RelayBuilderHiddenService {
    /// New tor hidden service options
    ///
    /// The nickname is a local identifier for a Tor hidden service.
    /// It's used to look up this service's keys, state, configuration, etc., and distinguish them from other services.
    #[inline]
    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    pub fn new<S>(nickname: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            nickname: nickname.into(),
            custom_path: None,
        }
    }

    /// New tor hidden service options
    ///
    /// The nickname is a local identifier for a Tor hidden service.
    /// It's used to look up this service's keys, state, configuration, etc., and distinguish them from other services.
    #[inline]
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub fn new<S, P>(nickname: S, path: P) -> Self
    where
        S: Into<String>,
        P: AsRef<Path>,
    {
        Self {
            nickname: nickname.into(),
            custom_path: Some(path.as_ref().to_path_buf()),
        }
    }
}

/// Mode
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayBuilderMode {
    /// Generic mode
    #[default]
    Generic,
    /// Accept only events that are authored by or contains a specific public key
    ///
    /// All other events are rejected
    PublicKey(PublicKey),
}

pub enum PolicyResult {
    /// Policy enforces that the event should be accepted
    Accept,
    /// Policy enforces that the event should be rejected
    Reject(String),
}

/// Event admission handler
#[async_trait]
pub trait WritePolicy: std::fmt::Debug + Send + Sync {
    async fn admit_event(&self, event: &Event, addr: &SocketAddr) -> PolicyResult;
}

#[async_trait]
pub trait QueryPolicy: std::fmt::Debug + Send + Sync {
    async fn admit_query(&self, query: &Vec<Filter>, addr: &SocketAddr) -> PolicyResult;
}

/// Relay builder
pub struct RelayBuilder {
    /// IP address
    pub addr: Option<IpAddr>,
    /// Port
    pub port: Option<u16>,
    /// Database
    pub database: Arc<DynNostrDatabase>,
    /// Mode
    pub mode: RelayBuilderMode,
    /// Rate limit
    pub rate_limit: RateLimit,
    /// Tor hidden service
    #[cfg(feature = "tor")]
    pub tor: Option<RelayBuilderHiddenService>,
    /// Max connections allowed
    pub max_connections: Option<usize>,
    /// Min POW difficulty
    pub min_pow: Option<u8>,
    /// Write policy plugins
    pub write_plugins: Vec<Arc<Box<dyn WritePolicy>>>,
    /// Query policy plugins
    pub query_plugins: Vec<Arc<Box<dyn QueryPolicy>>>,
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
            mode: RelayBuilderMode::default(),
            rate_limit: RateLimit::default(),
            #[cfg(feature = "tor")]
            tor: None,
            max_connections: None,
            min_pow: None,
            write_plugins: Vec::new(),
            query_plugins: Vec::new(),
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

    /// Set mode
    #[inline]
    pub fn mode(mut self, mode: RelayBuilderMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set rate limit
    #[inline]
    pub fn rate_limit(mut self, limit: RateLimit) -> Self {
        self.rate_limit = limit;
        self
    }

    /// Set tor options
    #[inline]
    #[cfg(feature = "tor")]
    pub fn tor(mut self, opts: RelayBuilderHiddenService) -> Self {
        self.tor = Some(opts);
        self
    }

    /// Set number of max connections allowed
    #[inline]
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// Set min POW difficulty
    #[inline]
    pub fn min_pow(mut self, difficulty: u8) -> Self {
        self.min_pow = Some(difficulty);
        self
    }

    /// Add a write policy plugin
    #[inline]
    pub fn write_policy<T: WritePolicy + 'static>(mut self, policy: T) -> Self {
        self.write_plugins.push(Arc::new(Box::new(policy)));
        self
    }

    /// Add a query policy plugin
    #[inline]
    pub fn query_policy<T: QueryPolicy + 'static>(mut self, policy: T) -> Self {
        self.query_plugins.push(Arc::new(Box::new(policy)));
        self
    }
}
