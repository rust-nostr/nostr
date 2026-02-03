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
#[must_use = "Does nothing unless you await!"]
pub struct GetRelays<'client> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::relays DOC.
    // --------------------------------------------------
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_client() -> Client {
        let client = Client::default();

        // Default capabilities (READ and WRITE)
        client.add_relay("wss://relay1.example.com").await.unwrap();
        client.add_relay("wss://relay2.example.com").await.unwrap();

        // Discovery capability
        client
            .add_relay("wss://relay3.example.com")
            .capabilities(RelayCapabilities::DISCOVERY)
            .await
            .unwrap();

        // Gossip capability
        client
            .add_relay("wss://relay4.example.com")
            .capabilities(RelayCapabilities::GOSSIP)
            .await
            .unwrap();

        client
    }

    #[tokio::test]
    async fn test_get_relays() {
        let client = setup_client().await;

        let url1 = RelayUrl::parse("wss://relay1.example.com").unwrap();
        let url2 = RelayUrl::parse("wss://relay2.example.com").unwrap();

        // By default, gets the relays with READ and WRITE capabilities
        let relays = client.relays().await;
        assert_eq!(relays.len(), 2);
        assert!(relays.contains_key(&url1));
        assert!(relays.contains_key(&url2));
    }

    #[tokio::test]
    async fn test_get_relays_with_capabilities() {
        let client = setup_client().await;

        let url3 = RelayUrl::parse("wss://relay3.example.com").unwrap();
        let url4 = RelayUrl::parse("wss://relay4.example.com").unwrap();

        // With gossip capability
        let relays = client
            .relays()
            .with_capabilities(RelayCapabilities::GOSSIP)
            .await;
        assert_eq!(relays.len(), 1);
        assert!(relays.contains_key(&url4));

        // Both gossip and discovery capabilities
        let relays = client
            .relays()
            .with_capabilities(RelayCapabilities::GOSSIP | RelayCapabilities::DISCOVERY)
            .await;
        assert_eq!(relays.len(), 2);
        assert!(relays.contains_key(&url3));
        assert!(relays.contains_key(&url4));
    }

    #[tokio::test]
    async fn test_get_all_relays() {
        let client = setup_client().await;

        let url1 = RelayUrl::parse("wss://relay1.example.com").unwrap();
        let url2 = RelayUrl::parse("wss://relay2.example.com").unwrap();
        let url3 = RelayUrl::parse("wss://relay3.example.com").unwrap();
        let url4 = RelayUrl::parse("wss://relay4.example.com").unwrap();

        // Get all
        let relays = client.relays().all().await;
        assert_eq!(relays.len(), 4);
        assert!(relays.contains_key(&url1));
        assert!(relays.contains_key(&url2));
        assert!(relays.contains_key(&url3));
        assert!(relays.contains_key(&url4));
    }
}
