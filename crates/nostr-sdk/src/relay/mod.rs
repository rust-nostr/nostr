// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nostr::{ClientMessage, Event, Filter, RelayMessage, SubscriptionId, Url};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

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

/// Relay instance's actual subscription with its unique id
#[derive(Debug, Clone)]
pub struct ActiveSubscription {
    /// SubscriptionId to update or cancel subscription
    pub id: SubscriptionId,
    /// Subscriptions filters
    pub filters: Vec<Filter>,
}

impl Default for ActiveSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveSubscription {
    /// Create new [`ActiveSubscription`]
    pub fn new() -> Self {
        Self {
            id: SubscriptionId::generate(),
            filters: Vec::new(),
        }
    }
}

/// Relay Pool Message
#[derive(Debug)]
pub enum RelayPoolMessage {
    /// Received new message
    ReceivedMsg {
        /// Relay url
        relay_url: Url,
        /// Relay message
        msg: RelayMessage,
    },
    /// Event sent
    EventSent(Box<Event>),
    /// Shutdown
    Shutdown,
}

/// Relay Pool Notification
#[derive(Debug, Clone)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]
    Event(Url, Event),
    /// Received a [`RelayMessage`]
    Message(Url, RelayMessage),
    /// Shutdown
    Shutdown,
}
