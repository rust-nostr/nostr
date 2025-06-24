// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Android Signer error

use std::fmt;

use jni::errors::Error as JniError;
use nostr::event;

/// Android Signer error
#[derive(Debug)]
pub enum Error {
    /// JNI error
    Jni(JniError),
    /// Nostr event error
    Event(event::Error),
    /// Can't find the JVM
    JVMNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jni(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::JVMNotFound => write!(f, "JVM not found"),
        }
    }
}

impl From<JniError> for Error {
    fn from(e: JniError) -> Self {
        Self::Jni(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}
