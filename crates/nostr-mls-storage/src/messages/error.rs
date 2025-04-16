//! Error types for the messages module

use std::fmt;

/// Error types for the messages module
#[derive(Debug)]
pub enum MessageError {
    /// Invalid parameters
    InvalidParameters(String),
    /// Database error
    DatabaseError(String),
    /// Message not found
    NotFound,
}

impl std::error::Error for MessageError {}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
            Self::NotFound => write!(f, "Message not found"),
        }
    }
}
