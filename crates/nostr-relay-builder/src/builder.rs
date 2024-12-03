// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Builder

use std::net::IpAddr;
#[cfg(all(feature = "tor", any(target_os = "android", target_os = "ios")))]
use std::path::Path;
#[cfg(feature = "tor")]
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

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
#[derive(Debug)]
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

/// Testing options
#[derive(Debug, Clone, Default)]
pub struct RelayTestOptions {
    /// Simulate unresponsive connection
    pub unresponsive_connection: Option<Duration>,
}

/// NIP42 mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayBuilderNip42Mode {
    /// Require authentication for writing
    Write,
    /// Require authentication for reading
    Read,
    /// Always require authentication
    #[default]
    Both,
}

impl RelayBuilderNip42Mode {
    /// Check if is [`RelayBuilderNip42Mode::Read`] or [`RelayBuilderNip42Mode::Both`]
    #[inline]
    pub fn is_read(&self) -> bool {
        matches!(self, Self::Read | Self::Both)
    }

    /// Check if is [`RelayBuilderNip42Mode::Write`] or [`RelayBuilderNip42Mode::Both`]
    #[inline]
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Write | Self::Both)
    }
}

/// NIP42 options
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelayBuilderNip42 {
    /// Mode
    pub mode: RelayBuilderNip42Mode,
    // /// Allowed public keys
    // pub allowed: HashSet<PublicKey>,
}

/// Relay builder
#[derive(Debug)]
pub struct RelayBuilder {
    /// IP address
    pub(crate) addr: Option<IpAddr>,
    /// Port
    pub(crate) port: Option<u16>,
    /// Database
    pub(crate) database: Arc<dyn NostrEventsDatabase>,
    /// Mode
    pub(crate) mode: RelayBuilderMode,
    /// Rate limit
    pub(crate) rate_limit: RateLimit,
    /// NIP42 options
    pub(crate) nip42: Option<RelayBuilderNip42>,
    /// Tor hidden service
    #[cfg(feature = "tor")]
    pub(crate) tor: Option<RelayBuilderHiddenService>,
    /// Max connections allowed
    pub(crate) max_connections: Option<usize>,
    /// Min POW difficulty
    pub(crate) min_pow: Option<u8>,
    /// Test options
    pub(crate) test: RelayTestOptions,
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
            nip42: None,
            #[cfg(feature = "tor")]
            tor: None,
            max_connections: None,
            min_pow: None,
            test: RelayTestOptions::default(),
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
        D: IntoNostrEventsDatabase,
    {
        self.database = database.into_database();
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

    /// Require NIP42 authentication
    #[inline]
    pub fn nip42(mut self, opts: RelayBuilderNip42) -> Self {
        self.nip42 = Some(opts);
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

    /// Testing options
    #[inline]
    pub(crate) fn test(mut self, test: RelayTestOptions) -> Self {
        self.test = test;
        self
    }
}
