use std::collections::HashMap;
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::RelayUrl;

use crate::client::Client;
use crate::relay::{Relay, RelayCapabilities};

enum Policy {
    All,
    WithCapabilities(RelayCapabilities),
}

/// Get relays
pub struct GetRelays<'client> {
    client: &'client Client,
    policy: Policy,
}

impl<'client> GetRelays<'client> {
    pub(crate) fn new(client: &'client Client) -> Self {
        Self {
            client,
            policy: Policy::WithCapabilities(RelayCapabilities::READ | RelayCapabilities::WRITE),
        }
    }

    /// Get all the relays in the pool.
    ///
    /// This method returns all relays added to the pool.
    ///
    /// This overwrites the current get policy!
    #[inline]
    pub fn all(mut self) -> Self {
        self.policy = Policy::All;
        self
    }

    /// Get relays that have any of the specified [`RelayCapabilities`].
    ///
    /// This overwrites the current get policy!
    #[inline]
    pub fn with_capabilities(mut self, capabilities: RelayCapabilities) -> Self {
        self.policy = Policy::WithCapabilities(capabilities);
        self
    }
}

impl<'client> IntoFuture for GetRelays<'client> {
    type Output = HashMap<RelayUrl, Relay>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self.policy {
                Policy::All => self.client.pool.all_relays().await,
                Policy::WithCapabilities(capabilities) => {
                    self.client.pool.relays_with_any_cap(capabilities).await
                }
            }
        })
    }
}

impl_blocking!(GetRelays<'_>);
