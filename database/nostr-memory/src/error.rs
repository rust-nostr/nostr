//! Memory Database error

use core::fmt;

/// Memory database error
#[derive(Debug)]
pub struct Error {
    _priv: (),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory database error")
    }
}
