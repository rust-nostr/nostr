use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use crate::blocking::Blocking;
use crate::policy::AdmitStatus;
use crate::relay::{Error, Relay, RelayStatus};
use crate::transport::websocket::{WebSocketSink, WebSocketStream};

/// Try to connect relay
#[must_use = "Does nothing unless you await!"]
pub struct TryConnect<'relay> {
    relay: &'relay Relay,
    timeout: Duration,
}

impl<'relay> TryConnect<'relay> {
    #[inline]
    pub(crate) fn new(relay: &'relay Relay) -> Self {
        Self {
            relay,
            timeout: Duration::from_secs(120),
        }
    }

    /// Timeout (default: 120 sec)
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn exec(self) -> Result<(), Error> {
        let status: RelayStatus = self.relay.status();

        if status.is_shutdown() {
            return Err(Error::Shutdown);
        }

        if status.is_banned() {
            return Err(Error::Banned);
        }

        // Check if relay can't connect
        if !status.can_connect() {
            return Ok(());
        }

        // Check connection policy
        if let AdmitStatus::Rejected { reason } = self.relay.inner.check_connection_policy().await?
        {
            // Set status to "terminated"
            self.relay.inner.set_status(RelayStatus::Terminated, false);

            // Return error
            return Err(Error::ConnectionRejected { reason });
        }

        // Try to connect
        // This will set the status to "terminated" if the connection fails
        let stream: (WebSocketSink, WebSocketStream) = self
            .relay
            .inner
            ._try_connect(self.timeout, RelayStatus::Terminated)
            .await?;

        // Spawn connection task
        self.relay.inner.spawn_connection_task(Some(stream));

        Ok(())
    }
}

impl<'relay> IntoFuture for TryConnect<'relay> {
    type Output = Result<(), Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'relay>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl Blocking for TryConnect<'_> {}

#[cfg(test)]
mod tests {
    use async_utility::time;
    use nostr::RelayUrl;
    use nostr_relay_builder::prelude::*;

    use super::{Error, *};

    #[tokio::test]
    async fn test_try_connect() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let relay: Relay = Relay::new(url);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay
            .try_connect()
            .timeout(Duration::from_millis(500))
            .await
            .unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        time::sleep(Duration::from_millis(500)).await;

        assert!(relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_try_connect_to_unreachable_relay() {
        let url = RelayUrl::parse("wss://127.0.0.1:666").unwrap();

        let relay: Relay = Relay::new(url);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        let res = relay.try_connect().timeout(Duration::from_secs(2)).await;
        assert!(matches!(res.unwrap_err(), Error::Transport(..)));

        assert_eq!(relay.status(), RelayStatus::Terminated);

        // Connection failed, the connection task is not running
        assert!(!relay.inner.is_running());
    }
}
