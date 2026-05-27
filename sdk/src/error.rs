// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! SDK error.

use std::error;

use nostr::{RelayUrl, serde_json};

opaquerr::define_kind! {
    /// SDK error kind.
    pub ErrorKind {
        /// Nostr protocol error.
        Protocol => "nostr protocol error",
        /// Transport error.
        Transport => "transport error",
        /// Database error.
        Database => "database error",
        /// Gossip error.
        Gossip => "gossip error",
        /// Policy error.
        Policy => "policy error",
        /// The operation timed out.
        Timeout => "timeout",
        /// Required data was not found.
        NotFound => "not found",
        /// Input is well-formed, but violates an SDK invariant.
        Invalid => "input violates an SDK invariant",
        /// The operation cannot be completed in the current state.
        State => "invalid state",
        /// The operation was rejected.
        Rejected => "operation rejected",
        /// The operation is known but not supported.
        Unsupported => "operation not supported",
        /// A configured limit was exceeded.
        LimitExceeded => "limit exceeded",
        /// Anything not covered by the stable categories above.
        Other => "other error",
    }
}

opaquerr::define_error! {
    /// SDK error.
    pub Error(ErrorKind)

    from {
        nostr::error::Error => ErrorKind::Protocol,
        nostr_database::error::Error => ErrorKind::Database,
        nostr_gossip::error::GossipError => ErrorKind::Gossip,
        serde_json::Error => ErrorKind::Protocol,
        faster_hex::Error => ErrorKind::Protocol,
        negentropy::Error => ErrorKind::Protocol,
        tokio::sync::oneshot::error::RecvError => ErrorKind::Other,
        tokio::sync::broadcast::error::RecvError => ErrorKind::Other,
    }
}

impl Error {
    /// Transport error.
    #[inline]
    pub fn transport<E>(error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Transport, error)
    }

    /// Policy error.
    #[inline]
    pub fn policy<E>(error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Policy, error)
    }

    #[inline]
    pub(crate) fn authentication_msg(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::Rejected, msg)
    }

    #[inline]
    pub(crate) fn gossip_not_configured() -> Self {
        Self::state_msg("gossip not configured")
    }

    /// Generic SDK error.
    #[inline]
    pub fn other<E>(error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Other, error)
    }

    #[inline]
    pub(crate) fn protocol_msg(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::Protocol, msg)
    }

    #[inline]
    pub(crate) fn invalid_msg(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::Invalid, msg)
    }

    #[inline]
    pub(crate) fn state_msg(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::State, msg)
    }

    #[inline]
    pub(crate) fn not_found(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::NotFound, msg)
    }

    #[inline]
    pub(crate) fn relay_not_found() -> Self {
        Self::with_static_message(ErrorKind::NotFound, "relay not found")
    }

    #[inline]
    pub(crate) fn relay_not_found_with_url(url: &RelayUrl) -> Self {
        Self::new(ErrorKind::NotFound, format!("relay '{url}' not found"))
    }

    #[inline]
    pub(crate) fn relays_not_specified() -> Self {
        Self::with_static_message(ErrorKind::Invalid, "relay/s not specified")
    }

    #[inline]
    pub(crate) fn limit_exceeded(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::LimitExceeded, msg)
    }

    #[inline]
    pub(crate) fn timeout() -> Self {
        Self::simple(ErrorKind::Timeout)
    }

    #[inline]
    pub(crate) fn not_connected() -> Self {
        Self::with_static_message(ErrorKind::State, "relay not connected")
    }

    #[inline]
    pub(crate) fn shutdown() -> Self {
        Self::with_static_message(ErrorKind::State, "shutdown")
    }

    #[inline]
    pub(crate) fn not_ready() -> Self {
        Self::with_static_message(ErrorKind::State, "relay is initialized but not ready")
    }

    #[inline]
    pub(crate) fn banned() -> Self {
        Self::with_static_message(ErrorKind::State, "relay banned")
    }

    #[inline]
    pub(crate) fn sleeping() -> Self {
        Self::with_static_message(ErrorKind::State, "relay is sleeping")
    }

    #[inline]
    pub(crate) fn read_disabled() -> Self {
        Self::with_static_message(ErrorKind::Unsupported, "read actions are disabled")
    }

    #[inline]
    pub(crate) fn write_disabled() -> Self {
        Self::with_static_message(ErrorKind::Unsupported, "write actions are disabled")
    }

    #[inline]
    pub(crate) fn rejected_msg(msg: &'static str) -> Self {
        Self::with_static_message(ErrorKind::Rejected, msg)
    }

    #[inline]
    pub(crate) fn rejected<E>(error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Rejected, error)
    }

    #[inline]
    pub(crate) fn relay_msg(msg: String) -> Self {
        Self::new(ErrorKind::Rejected, msg)
    }

    #[inline]
    pub(crate) fn connection_rejected(reason: Option<String>) -> Self {
        match reason {
            Some(reason) => Self::new(
                ErrorKind::Rejected,
                format!("connection rejected: reason={reason}"),
            ),
            None => Self::with_static_message(ErrorKind::Rejected, "connection rejected"),
        }
    }

    #[inline]
    pub(crate) fn negentropy_not_supported() -> Self {
        Self::with_static_message(ErrorKind::Unsupported, "negentropy not supported")
    }

    #[inline]
    pub(crate) fn unknown_negentropy_error() -> Self {
        Self::with_static_message(ErrorKind::Protocol, "unknown negentropy error")
    }

    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn pong_not_match(expected: u64, received: u64) -> Self {
        Self::new(
            ErrorKind::Protocol,
            format!("pong not match: expected={expected}, received={received}"),
        )
    }
}
