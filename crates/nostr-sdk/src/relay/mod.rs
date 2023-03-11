// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay Module

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nostr::ClientMessage;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::*;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;

/// Relay event
#[derive(Debug)]
pub enum RelayEvent {
    /// Send [`ClientMessage`]
    SendMsg(Box<ClientMessage>),
    // Ping,
    /// Close
    Close,
    /// Completly disconnect
    Terminate,
}

/// Relay connection status
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Relay connected
    Connected,
    /// Connecting
    Connecting,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Relay completly disconnected
    Terminated,
}

impl fmt::Display for RelayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized => write!(f, "Initialized"),
            Self::Connected => write!(f, "Connected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

/// [`Relay`] options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    /// Allow/disallow read actions
    read: Arc<AtomicBool>,
    /// Allow/disallow write actions
    write: Arc<AtomicBool>,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl RelayOptions {
    /// New [`RelayOptions`]
    pub fn new(read: bool, write: bool) -> Self {
        Self {
            read: Arc::new(AtomicBool::new(read)),
            write: Arc::new(AtomicBool::new(write)),
        }
    }

    /// Get read option
    pub fn read(&self) -> bool {
        self.read.load(Ordering::SeqCst)
    }

    /// Set read option
    pub fn set_read(&self, read: bool) {
        let _ = self
            .read
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(read));
    }

    /// Get write option
    pub fn write(&self) -> bool {
        self.write.load(Ordering::SeqCst)
    }

    /// Set write option
    pub fn set_write(&self, write: bool) {
        let _ = self
            .write
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(write));
    }
}
