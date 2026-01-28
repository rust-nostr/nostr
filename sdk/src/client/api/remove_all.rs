use std::future::{Future, IntoFuture};
use std::pin::Pin;

use crate::blocking::Blocking;
use crate::client::{Client, Error};

/// Remove all relays from the pool.
#[must_use = "Does nothing unless you await!"]
pub struct RemoveAllRelays<'client> {
    client: &'client Client,
    force: bool,
}

impl<'client> RemoveAllRelays<'client> {
    pub(crate) fn new(client: &'client Client) -> Self {
        Self {
            client,
            force: false,
        }
    }

    /// Force remove all the relays
    #[inline]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    #[inline]
    async fn exec(self) -> Result<(), Error> {
        self.client.pool.remove_all_relays(self.force).await;
        Ok(())
    }
}

impl<'client> IntoFuture for RemoveAllRelays<'client> {
    type Output = Result<(), Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl Blocking for RemoveAllRelays<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::RelayCapabilities;

    #[tokio::test]
    async fn test_remove_all_relays() {
        let client = Client::default();

        client.add_relay("ws://127.0.0.1:6666").await.unwrap();

        client.add_relay("ws://127.0.0.1:7777").await.unwrap();

        client
            .add_relay("ws://127.0.0.1:8888")
            .capabilities(RelayCapabilities::default() | RelayCapabilities::GOSSIP)
            .await
            .unwrap();

        assert_eq!(client.relays().await.len(), 3);
        assert_eq!(client.pool.all_relays().await.len(), 3);

        // Remove all relays
        client.remove_all_relays().await.unwrap();
        assert!(client.relay("ws://127.0.0.1:6666").await.unwrap().is_none());
        assert!(client.relay("ws://127.0.0.1:7777").await.unwrap().is_none());
        assert!(client.relay("ws://127.0.0.1:8888").await.is_ok()); // The GOSSIP relay still exists
        assert!(client.relays().await.is_empty()); // This gets only the READ/WRITE relays, which are now 0
        assert_eq!(client.pool.all_relays().await.len(), 1); // The GOSSIP relay still exists
    }

    #[tokio::test]
    async fn test_force_remove_all_relays() {
        let client = Client::default();

        client.add_relay("ws://127.0.0.1:6666").await.unwrap();

        client.add_relay("ws://127.0.0.1:7777").await.unwrap();

        client
            .add_relay("ws://127.0.0.1:8888")
            .capabilities(RelayCapabilities::default() | RelayCapabilities::GOSSIP)
            .await
            .unwrap();

        assert_eq!(client.relays().await.len(), 3);
        assert_eq!(client.pool.all_relays().await.len(), 3);

        // Force remove all relays
        client.remove_all_relays().force().await.unwrap();

        // Check if relays map is empty
        assert!(client.relays().await.is_empty());
        assert!(client.pool.all_relays().await.is_empty());

        // Double check that relays doesn't exist
        assert!(client.relay("ws://127.0.0.1:6666").await.unwrap().is_none());
        assert!(client.relay("ws://127.0.0.1:7777").await.unwrap().is_none());
        assert!(client.relay("ws://127.0.0.1:8888").await.unwrap().is_none());
    }
}
