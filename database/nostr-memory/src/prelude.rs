//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

pub use nostr::prelude::*;
pub use nostr_database::prelude::*;

pub use crate::builder::{self, *};
pub use crate::error::{self, *};
pub use crate::*;
