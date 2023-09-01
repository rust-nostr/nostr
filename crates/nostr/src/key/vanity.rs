// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Vanity

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, RecvError};
use std::thread;

use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::SecretKey;

use super::Keys;
use crate::nips::nip19::{ToBech32, PREFIX_BECH32_PUBLIC_KEY};
use crate::SECP256K1;

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
    /// Generate new vanity public key
    pub fn vanity<S>(prefixes: Vec<S>, bech32: bool, num_cores: usize) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let prefixes: Vec<String> = prefixes.into_iter().map(|p| p.into()).collect();

        if bech32 {
            for prefix in prefixes.iter() {
                for c in prefix.chars() {
                    if !BECH32_CHARS.contains(c) {
                        return Err(Error::InvalidChar(c));
                    }
                }
            }
        } else {
            for prefix in prefixes.iter() {
                for c in prefix.chars() {
                    if !HEX_CHARS.contains(c) {
                        return Err(Error::InvalidChar(c));
                    }
                }
            }
        }

        let (tx, rx) = sync_channel::<SecretKey>(1);
        let found = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();

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

                    let (secret_key, public_key) = SECP256K1.generate_keypair(&mut rng);
                    let (xonly_public_key, _) = public_key.x_only_public_key();

                    if bech32 {
                        let bech32_key = xonly_public_key
                            .to_bech32()
                            .expect("Unable to convert key to bech32");
                        if prefixes.iter().any(|prefix| {
                            bech32_key.starts_with(&format!("{PREFIX_BECH32_PUBLIC_KEY}1{prefix}"))
                        }) {
                            tx.send(secret_key).expect("Unable to send on channel");
                            let _ = found
                                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(true));
                            break;
                        }
                    } else {
                        let pubkey = xonly_public_key.to_string();
                        if prefixes.iter().any(|prefix| pubkey.starts_with(prefix)) {
                            tx.send(secret_key).expect("Unable to send on channel");
                            let _ = found
                                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(true));
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

        Ok(Self::new(rx.recv()?))
    }
}
