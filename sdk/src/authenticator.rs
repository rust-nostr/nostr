//! A NIP-42 authenticator

use std::any::Any;
use std::fmt::Debug;

use nostr::signer::{AsyncGetPublicKey, AsyncSignEvent};
use nostr::{Event, EventBuilder, RelayUrl};

use crate::future::BoxedFuture;

/// Authenticator error
pub type AuthenticationError = Box<dyn std::error::Error + Send + Sync>;

/// Authenticator
pub trait Authenticator: Any + Debug + Send + Sync {
    /// Makes a NIP-42 event for authentication
    ///
    /// Must return a valid NIP-42 event.
    fn make_auth_event<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
        challenge: &'a str,
    ) -> BoxedFuture<'a, Result<Event, AuthenticationError>>;
}

/// An authenticator that uses a signer that implements [`AsyncGetPublicKey`] and [`AsyncSignEvent`] for creating NIP-42 events.
#[derive(Debug)]
pub struct SignerAuthenticator<T>
where
    T: AsyncGetPublicKey + AsyncSignEvent,
{
    signer: T,
}

impl<T> SignerAuthenticator<T>
where
    T: AsyncGetPublicKey + AsyncSignEvent,
{
    /// Constructs a new authenticator
    #[inline]
    pub fn new(signer: T) -> Self {
        Self { signer }
    }
}

impl<T> From<T> for SignerAuthenticator<T>
where
    T: AsyncGetPublicKey + AsyncSignEvent,
{
    /// Constructs a new authenticator from a signer
    #[inline]
    fn from(signer: T) -> Self {
        Self::new(signer)
    }
}

impl<T> Authenticator for SignerAuthenticator<T>
where
    T: AsyncGetPublicKey + AsyncSignEvent,
{
    fn make_auth_event<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
        challenge: &'a str,
    ) -> BoxedFuture<'a, Result<Event, AuthenticationError>> {
        Box::pin(async move {
            Ok(EventBuilder::auth(challenge, relay_url.clone())
                .sign_async(&self.signer)
                .await?)
        })
    }
}
