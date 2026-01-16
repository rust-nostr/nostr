use std::borrow::Cow;
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::types::url::{RelayUrl, RelayUrlArg};

use super::blocking::Blocking;
use crate::client::{Client, Error};

/// Remove a relay from the pool.
#[must_use = "Does nothing unless you await!"]
pub struct RemoveRelay<'client, 'url> {
    client: &'client Client,
    url: RelayUrlArg<'url>,
    force: bool,
}

impl<'client, 'url> RemoveRelay<'client, 'url> {
    pub(crate) fn new(client: &'client Client, url: RelayUrlArg<'url>) -> Self {
        Self {
            client,
            url,
            force: false,
        }
    }

    /// Force remove
    #[inline]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    async fn exec(self) -> Result<(), Error> {
        // Convert into relay URL
        let url: Cow<RelayUrl> = self.url.try_into_relay_url()?;

        // Remove the relay from the pool
        Ok(self.client.pool.remove_relay(url, self.force).await?)
    }
}

impl<'client, 'url> IntoFuture for RemoveRelay<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<(), Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl<'client, 'url> Blocking for RemoveRelay<'client, 'url> where 'url: 'client {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool;
    use crate::relay::capabilities::RelayCapabilities;

    #[tokio::test]
    async fn test_remove_nonexistent_relay() {
        let client = Client::default();

        client.add_relay("ws://127.0.0.1:6666").await.unwrap();

        assert!(matches!(
            client
                .remove_relay("ws://127.0.0.1:7777")
                .await
                .unwrap_err(),
            Error::RelayPool(pool::Error::RelayNotFound)
        ));
    }

    #[tokio::test]
    async fn test_remove_relay() {
        let client = Client::default();

        client.add_relay("ws://127.0.0.1:6666").await.unwrap();

        client
            .add_relay("ws://127.0.0.1:8888")
            .capabilities(RelayCapabilities::default() | RelayCapabilities::GOSSIP)
            .await
            .unwrap();

        assert_eq!(client.relays().await.len(), 2);
        assert_eq!(client.pool.all_relays().await.len(), 2);

        // Remove the non-gossip relay
        assert!(client.remove_relay("ws://127.0.0.1:6666").await.is_ok());
        assert!(matches!(
            client.relay("ws://127.0.0.1:6666").await.unwrap_err(),
            Error::RelayPool(pool::Error::RelayNotFound)
        ));
        assert_eq!(client.relays().await.len(), 1);
        assert_eq!(client.pool.all_relays().await.len(), 1);

        // Try to remove the gossip relay (will not be removed)
        assert!(client.remove_relay("ws://127.0.0.1:8888").await.is_ok());
        assert!(client.relay("ws://127.0.0.1:8888").await.is_ok()); // The relay exists in the client!
        assert!(client.relays().await.is_empty()); // This gets only the READ/WRITE relays, which are now 0
        assert_eq!(client.pool.all_relays().await.len(), 1);
    }

    #[tokio::test]
    async fn test_force_remove_relay() {
        let client = Client::default();

        client.add_relay("ws://127.0.0.1:6666").await.unwrap();

        client
            .add_relay("ws://127.0.0.1:8888")
            .capabilities(RelayCapabilities::default() | RelayCapabilities::GOSSIP)
            .await
            .unwrap();

        assert_eq!(client.relays().await.len(), 2);
        assert_eq!(client.pool.all_relays().await.len(), 2);

        // Force remove the non-gossip relay
        assert!(client
            .remove_relay("ws://127.0.0.1:6666")
            .force()
            .await
            .is_ok());
        assert!(matches!(
            client.relay("ws://127.0.0.1:6666").await.unwrap_err(),
            Error::RelayPool(pool::Error::RelayNotFound)
        ));
        assert_eq!(client.relays().await.len(), 1);
        assert_eq!(client.pool.all_relays().await.len(), 1);

        // Force remove the gossip relay
        assert!(client
            .remove_relay("ws://127.0.0.1:8888")
            .force()
            .await
            .is_ok());
        assert!(matches!(
            client.relay("ws://127.0.0.1:8888").await.unwrap_err(),
            Error::RelayPool(pool::Error::RelayNotFound)
        ));
        assert!(client.relays().await.is_empty());
        assert!(client.pool.all_relays().await.is_empty());
    }
}
