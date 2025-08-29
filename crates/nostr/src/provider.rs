//! Nostr provider

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::ops::Range;
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
use instant::SystemTime;
#[cfg(not(feature = "std"))]
use once_cell::race::OnceBox;
#[cfg(feature = "std")]
use once_cell::sync::OnceCell;
#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;
#[cfg(feature = "std")]
use secp256k1::rand::RngCore;
use secp256k1::{All, Secp256k1};

use crate::Timestamp;

#[cfg(target_arch = "wasm32")]
const UNIX_EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

#[cfg(feature = "std")]
static NOSTR_PROVIDER: OnceCell<NostrProvider> = OnceCell::new();
#[cfg(not(feature = "std"))]
static NOSTR_PROVIDER: OnceBox<NostrProvider> = OnceBox::new();

/// Helper trait for acquiring time in `no_std` environments.
pub trait TimeProvider: Debug + Send + Sync {
    /// Get the current UNIX timestamp.
    fn now(&self) -> Timestamp;
}

/// Default time provider.
#[derive(Debug)]
pub struct DefaultTimeProvider;

#[cfg(feature = "std")]
impl TimeProvider for DefaultTimeProvider {
    fn now(&self) -> Timestamp {
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Timestamp::from_secs(ts)
    }
}

/// A source of cryptographically secure randomness.
pub trait SecureRandom: Debug + Send + Sync {
    /// Fill the given buffer with random bytes.
    ///
    /// The bytes must be sourced from a cryptographically secure random number
    /// generator seeded with good quality, secret entropy.
    fn fill(&self, buf: &mut [u8]);

    /// Generate a random u64 in range.
    fn gen_range_u64(&self, range: Range<u64>) -> u64 {
        let range_size: u64 = range.end - range.start;
        let mut bytes: [u8; 8] = [0u8; 8];
        self.fill(&mut bytes);
        let random: u64 = u64::from_le_bytes(bytes);
        range.start + (random % range_size)
    }
}

#[cfg(feature = "std")]
impl SecureRandom for OsRng {
    fn fill(&self, buf: &mut [u8]) {
        OsRng.fill_bytes(buf);
    }
}

// TODO: use another name?
/// Nostr provider
#[derive(Debug)]
pub struct NostrProvider {
    /// Secp256k1 context
    pub secp: Secp256k1<All>,
    /// Time provider
    pub time: Arc<dyn TimeProvider>,
    /// Secure random number generator
    pub rng: Arc<dyn SecureRandom>,
}

#[cfg(feature = "std")]
impl Default for NostrProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl NostrProvider {
    /// New default nostr provider
    ///
    /// This set up the provider as following:
    /// - Secp256k1 context, randomized with randomness from the operating system;
    /// - Time provider, using the [`SystemTime`];
    /// - Secure random number generator, using [`OsRng`].
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        let mut rng = OsRng;

        let mut secp = Secp256k1::new();
        secp.randomize(&mut rng);

        Self {
            secp,
            time: Arc::new(DefaultTimeProvider),
            rng: Arc::new(rng),
        }
    }

    /// Install provider
    ///
    /// # Panic
    ///
    /// Panic if a provider is already installed!
    /// Use [`NostrProvider::try_install()`] for a non-panic version of this method.
    #[inline]
    pub fn install(self) {
        self.try_install()
            .expect("Nostr provider already installed!");
    }

    /// Try to install the provider
    ///
    /// Returns `Err(<installed-provider>)` if a provider was already installed.
    pub fn try_install(self) -> Result<(), Self> {
        #[cfg(feature = "std")]
        let this: Self = self;

        #[cfg(not(feature = "std"))]
        let this = Box::new(self);

        NOSTR_PROVIDER.set(this)
    }

    #[inline]
    #[cfg(feature = "std")]
    pub(crate) fn get() -> &'static Self {
        NOSTR_PROVIDER.get_or_init(Self::new)
    }

    #[inline]
    #[cfg(not(feature = "std"))]
    pub(crate) fn get() -> &'static Self {
        NOSTR_PROVIDER
            .get()
            .expect("Nostr provider not initialized! Please call `NostrProvider::install()` first.")
    }
}
