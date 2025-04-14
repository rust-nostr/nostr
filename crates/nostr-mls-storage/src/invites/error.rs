use std::fmt;

pub enum InviteError {
    InvalidParameters(String),
    NotFound,
    DatabaseError(String),
}

impl std::error::Error for InviteError {}

impl fmt::Display for InviteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::NotFound => write!(f, "Invite not found"),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
        }
    }
}

impl fmt::Debug for InviteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
