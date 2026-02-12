use std::borrow::Cow;
use std::future::IntoFuture;
use std::time::Duration;

use async_wsocket::ConnectionMode;
use nostr::types::url::{RelayUrl, RelayUrlArg};

use crate::client::{Client, Error};
use crate::future::BoxedFuture;
use crate::relay::{RelayCapabilities, RelayLimits, RelayOptions};

/// Add new relay to the pool
#[must_use = "Does nothing unless you await!"]
pub struct AddRelay<'client, 'url> {
    client: &'client Client,
    url: RelayUrlArg<'url>,
    capabilities: RelayCapabilities,
    connect: bool,
    opts: RelayOptions,
}

impl<'client, 'url> AddRelay<'client, 'url> {
    pub(crate) fn new(client: &'client Client, url: RelayUrlArg<'url>) -> Self {
        Self {
            client,
            url,
            capabilities: RelayCapabilities::default(),
            connect: false,
            opts: RelayOptions::default(),
        }
    }

    /// Set capabilities
    ///
    /// If the relay already exists, the capabilities will be added to the existing one.
    #[inline]
    pub fn capabilities(mut self, capabilities: RelayCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Connection timeout (default: 15 sec)
    ///
    /// This is the default timeout use when attempting to establish a connection with the relay
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.opts.connect_timeout = timeout;
        self
    }

    /// Set connection mode
    #[inline]
    pub fn connection_mode(mut self, mode: ConnectionMode) -> Self {
        self.opts.connection_mode = mode;
        self
    }

    /// Enable or disable ping
    #[inline]
    pub fn ping(mut self, enable: bool) -> Self {
        self.opts.ping = enable;
        self
    }

    /// Enable/disable auto reconnection (default: true)
    pub fn reconnect(mut self, reconnect: bool) -> Self {
        self.opts.reconnect = reconnect;
        self
    }

    /// Retry connection time (default: 10 sec)
    pub fn retry_interval(mut self, interval: Duration) -> Self {
        self.opts.retry_interval = interval;
        self
    }

    /// Automatically adjust retry interval based on success/attempts (default: true)
    pub fn adjust_retry_interval(mut self, adjust_retry_interval: bool) -> Self {
        self.opts.adjust_retry_interval = adjust_retry_interval;
        self
    }

    /// Verify that received events belong to a subscription and match the filter.
    pub fn verify_subscriptions(mut self, enable: bool) -> Self {
        self.opts.verify_subscriptions = enable;
        self
    }

    /// If true, ban a relay when it sends an event that doesn't match the subscription filter.
    pub fn ban_relay_on_mismatch(mut self, ban_relay: bool) -> Self {
        self.opts.ban_relay_on_mismatch = ban_relay;
        self
    }

    /// Set custom limits
    pub fn limits(mut self, limits: RelayLimits) -> Self {
        self.opts.limits = limits;
        self
    }

    /// Set max latency (default: None)
    ///
    /// Relay with an avg. latency greater that this value will be skipped.
    #[inline]
    pub fn max_avg_latency(mut self, max: Option<Duration>) -> Self {
        self.opts.max_avg_latency = max;
        self
    }

    /// Notification channel size (default: 4096)
    #[inline]
    pub fn notification_channel_size(mut self, size: usize) -> Self {
        self.opts.notification_channel_size = size;
        self
    }

    /// Sleep when idle (default: false)
    #[inline]
    pub fn sleep_when_idle(mut self, enable: bool) -> Self {
        self.opts.sleep_when_idle = enable;
        self
    }

    /// Set idle timeout for on-demand connections (default: 5 minutes)
    #[inline]
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.opts.idle_timeout = timeout;
        self
    }

    /// Connect to the relay after adding it to the client
    #[inline]
    pub fn and_connect(mut self) -> Self {
        self.connect = true;
        self
    }

    /// Set relay options.
    ///
    /// **Warning**: this method overrides any previously set options.
    #[inline]
    pub fn opts(mut self, opts: RelayOptions) -> Self {
        self.opts = opts;
        self
    }
}

impl<'client, 'url> IntoFuture for AddRelay<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<bool, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Convert into relay URL
            let url: Cow<RelayUrl> = self.url.try_into_relay_url()?;

            // Add relay to the pool
            Ok(self
                .client
                .pool
                .add_relay(url, self.capabilities, self.connect, self.opts)
                .await?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_relay() {
        let client = Client::default();

        let res = client.add_relay("wss://relay.damus.io").await.unwrap();
        assert!(res);

        // Try to re-add it
        let res = client.add_relay("wss://relay.damus.io").await.unwrap();
        assert!(!res);
    }

    #[tokio::test]
    async fn test_add_relay_default_capabilities() {
        let client = Client::default();

        // Add relay
        let res = client.add_relay("wss://relay.damus.io").await.unwrap();
        assert!(res);

        // Verify capabilities
        let relay = client.relay("wss://relay.damus.io").await.unwrap().unwrap();
        assert_eq!(
            relay.capabilities().load(),
            RelayCapabilities::READ | RelayCapabilities::WRITE
        );
    }

    #[tokio::test]
    async fn test_add_relay_with_capability() {
        let client = Client::default();

        // Add relay with READ capability
        let res = client
            .add_relay("wss://relay.damus.io")
            .capabilities(RelayCapabilities::READ)
            .await
            .unwrap();
        assert!(res);

        // Verify capabilities
        let relay = client.relay("wss://relay.damus.io").await.unwrap().unwrap();
        assert_eq!(relay.capabilities().load(), RelayCapabilities::READ);

        // Try to re-add relay with GOSSIP capability
        let res = client
            .add_relay("wss://relay.damus.io")
            .capabilities(RelayCapabilities::GOSSIP)
            .await
            .unwrap();
        assert!(!res); // Already exists, so must return false

        // Verify capabilities
        let relay = client.relay("wss://relay.damus.io").await.unwrap().unwrap();
        assert_eq!(
            relay.capabilities().load(),
            RelayCapabilities::READ | RelayCapabilities::GOSSIP
        );
    }
}
