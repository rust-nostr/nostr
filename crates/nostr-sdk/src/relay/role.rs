// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

/// Relay role
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayRole {
    /// Generic
    #[default]
    Generic,
    /// Identity
    Identity,
    /// Gossip
    Gossip,
    /// Custom
    Custom(String),
}

impl fmt::Display for RelayRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic => write!(f, "generic"),
            Self::Identity => write!(f, "identity"),
            Self::Gossip => write!(f, "gossip"),
            Self::Custom(c) => write!(f, "{c}"),
        }
    }
}
