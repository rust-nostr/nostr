//! Error types for the groups module

use std::fmt;

/// Invalid group state
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidGroupState {
    /// Group has no admins
    NoAdmins,
    /// Group has no relays
    NoRelays,
}

impl fmt::Display for InvalidGroupState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdmins => write!(f, "group has no admins"),
            Self::NoRelays => write!(f, "group has no relays"),
        }
    }
}

/// Error types for the groups module
#[derive(Debug)]
pub enum GroupError {
    /// Invalid parameters
    InvalidParameters(String),
    /// Database error
    DatabaseError(String),
    /// Invalid state
    InvalidState(InvalidGroupState),
}

impl std::error::Error for GroupError {}

impl fmt::Display for GroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
            Self::InvalidState(state) => write!(f, "Invalid state: {state}"),
        }
    }
}
