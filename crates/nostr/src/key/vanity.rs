// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Vanity

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, RecvError};
use std::thread;

use secp256k1::rand;

use super::Keys;
use crate::nips::nip19::{ToBech32, PREFIX_BECH32_PUBLIC_KEY};

const BECH32_SPAN: usize = PREFIX_BECH32_PUBLIC_KEY.len() + 1;
const BECH32_CHARS: &str = "023456789acdefghjklmnpqrstuvwxyz";
const HEX_CHARS: &str = "0123456789abcdef";

/// [`Keys`] vanity error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Unsupported char
    InvalidChar(char),
    /// RecvError
    RecvError(RecvError),
    /// Thread Join failed
    JoinHandleError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChar(c) => write!(f, "Unsupported char: {c}"),
            Self::RecvError(e) => write!(f, "{e}"),
            Self::JoinHandleError => write!(f, "impossible to join threads"),
        }
    }
}

impl From<RecvError> for Error {
    fn from(e: RecvError) -> Self {
        Self::RecvError(e)
    }
}

impl Keys {
    /// check validity of prefix characters
    fn check_prefix_chars(prefixes: &[String], valid_chars: &str) -> Result<(), Error> {
        for prefix in prefixes.iter() {
            for c in prefix.chars() {
                if !valid_chars.contains(c) {
                    return Err(Error::InvalidChar(c));
                }
            }
        }
        Ok(())
    }

    /// Generate new vanity public key
    #[deprecated(since = "0.39.0")]
    pub fn vanity<S>(prefixes: Vec<S>, bech32: bool, num_cores: usize) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let prefixes: Vec<String> = prefixes.into_iter().map(|p| p.into()).collect();
        if bech32 {
            Self::check_prefix_chars(&prefixes, BECH32_CHARS)?;
        } else {
            Self::check_prefix_chars(&prefixes, HEX_CHARS)?;
        }
        let (tx, rx) = sync_channel::<Keys>(1);
        let found = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::with_capacity(num_cores);

        for _ in 0..num_cores {
            let tx = tx.clone();
            let found = found.clone();
            let prefixes = prefixes.clone();
            let handle = thread::spawn(move || {
                let mut rng = rand::thread_rng();
                loop {
                    if found.load(Ordering::SeqCst) {
                        break;
                    }

                    let keys: Keys = Keys::generate_with_rng(&mut rng);

                    if bech32 {
                        let bech32_key = keys
                            .public_key
                            .to_bech32()
                            .expect("Unable to convert key to bech32");
                        if prefixes
                            .iter()
                            .any(|prefix| bech32_key[BECH32_SPAN..].starts_with(prefix))
                        {
                            tx.send(keys).expect("Unable to send on channel");
                            found.store(true, Ordering::SeqCst);
                            break;
                        }
                    } else {
                        let pubkey = keys.public_key.to_string();
                        if prefixes.iter().any(|prefix| pubkey.starts_with(prefix)) {
                            tx.send(keys).expect("Unable to send on channel");
                            found.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().map_err(|_| Error::JoinHandleError)?;
        }

        Ok(rx.recv()?)
    }
}
