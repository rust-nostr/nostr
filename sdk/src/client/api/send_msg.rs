use std::collections::HashSet;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use nostr::{ClientMessage, RelayUrl, RelayUrlArg};

use super::output::Output;
use crate::client::{Client, Error};
use crate::relay::RelayCapabilities;

enum OverwritePolicy<'url> {
    // All READ and WRITE relays
    Broadcast,
    // To specific relays
    To(Vec<RelayUrlArg<'url>>),
}

/// Send the client message
#[must_use = "Does nothing unless you await!"]
pub struct SendMessage<'client, 'msg, 'url> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::send_msg DOC.
    // --------------------------------------------------
    client: &'client Client,
    msg: ClientMessage<'msg>,
    policy: Option<OverwritePolicy<'url>>,
    wait_until_sent: Option<Duration>,
}

impl<'client, 'msg, 'url> SendMessage<'client, 'msg, 'url> {
    pub(crate) fn new(client: &'client Client, msg: ClientMessage<'msg>) -> Self {
        Self {
            client,
            msg,
            policy: None,
            wait_until_sent: None,
        }
    }

    /// Send the message to all relays with [`RelayCapabilities::READ`] and [`RelayCapabilities::WRITE`] capability.
    ///
    /// This overwrites the [`SendMessage::to`] method.
    #[inline]
    pub fn broadcast(mut self) -> Self {
        self.policy = Some(OverwritePolicy::Broadcast);
        self
    }

    /// Send the message to specific relays
    ///
    /// This overwrites the [`SendMessage::broadcast`] method.
    pub fn to<I, T>(mut self, urls: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<RelayUrlArg<'url>>,
    {
        self.policy = Some(OverwritePolicy::To(
            urls.into_iter().map(Into::into).collect(),
        ));
        self
    }

    /// Wait that message is sent
    #[inline]
    pub fn wait_until_sent(mut self, timeout: Duration) -> Self {
        self.wait_until_sent = Some(timeout);
        self
    }
}

impl<'client, 'msg, 'url> IntoFuture for SendMessage<'client, 'msg, 'url>
where
    'msg: 'client,
    'url: 'client,
{
    type Output = Result<Output<()>, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let urls: HashSet<RelayUrl> = match self.policy {
                Some(OverwritePolicy::Broadcast) | None => {
                    self.client
                        .pool
                        .relay_urls_with_any_cap(RelayCapabilities::READ | RelayCapabilities::WRITE)
                        .await
                }
                Some(OverwritePolicy::To(list)) => {
                    let mut urls: HashSet<RelayUrl> = HashSet::with_capacity(list.len());

                    for url in list {
                        let url: RelayUrl = url.try_into_relay_url()?.into_owned();
                        urls.insert(url);
                    }

                    urls
                }
            };

            Ok(self
                .client
                .pool
                .send_msg(urls, self.msg, self.wait_until_sent)
                .await?)
        })
    }
}

impl_blocking!(for<'client, 'msg, 'url> SendMessage<'client, 'msg, 'url> where 'msg: 'client, 'url: 'client);

#[cfg(test)]
mod tests {
    use nostr::prelude::*;
    use nostr_relay_builder::MockRelay;

    use super::*;

    #[tokio::test]
    async fn test_send_msg() {
        let mock1 = MockRelay::run().await.unwrap();
        let url1 = mock1.url().await;
        let mock2 = MockRelay::run().await.unwrap();
        let url2 = mock2.url().await;
        let mock3 = MockRelay::run().await.unwrap();
        let url3 = mock3.url().await;

        let client: Client = Client::default();

        // Add 2 READ and WRITE relays
        client.add_relay(&url1).await.unwrap();
        client.add_relay(&url2).await.unwrap();

        // Add a DISCOVERY relay
        client
            .add_relay(&url3)
            .capabilities(RelayCapabilities::DISCOVERY)
            .await
            .unwrap();

        client.connect().await;

        let msg = ClientMessage::req(SubscriptionId::new("test"), vec![Filter::new().limit(10)]);

        // Send msg (broadcast to all READ and WRITE relays by default)
        let output = client.send_msg(msg).await.unwrap();

        assert_eq!(output.success.len(), 2);
        assert!(output.success.contains(&url1));
        assert!(output.success.contains(&url2));
        assert!(!output.success.contains(&url3));
        assert!(output.failed.is_empty());
    }

    #[tokio::test]
    async fn test_send_msg_to() {
        let mock1 = MockRelay::run().await.unwrap();
        let url1 = mock1.url().await;
        let mock2 = MockRelay::run().await.unwrap();
        let url2 = mock2.url().await;

        let client = Client::default();
        client.add_relay(&url1).await.unwrap();
        client.add_relay(&url2).await.unwrap();
        client.connect().await;

        let msg = ClientMessage::req(SubscriptionId::new("test"), vec![Filter::new().limit(10)]);

        // Send only to relay 1
        let output = client.send_msg(msg).to([&url1]).await.unwrap();

        assert_eq!(output.success.len(), 1);
        assert!(output.success.contains(&url1));
        assert!(!output.success.contains(&url2));
        assert!(output.failed.is_empty());
    }
}
