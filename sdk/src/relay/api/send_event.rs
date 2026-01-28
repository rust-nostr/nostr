use std::borrow::Cow;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use async_utility::time;
use nostr::message::MachineReadablePrefix;
use nostr::{ClientMessage, Event, EventId};
use tokio::sync::broadcast;

use crate::blocking::Blocking;
use crate::relay::{Error, Relay, RelayNotification};

/// Send event to relay
#[must_use = "Does nothing unless you await!"]
pub struct SendEvent<'relay, 'event> {
    relay: &'relay Relay,
    event: &'event Event,
    wait_for_ok_timeout: Duration,
    wait_for_authentication_timeout: Duration,
}

impl<'relay, 'event> SendEvent<'relay, 'event> {
    pub(crate) fn new(relay: &'relay Relay, event: &'event Event) -> Self {
        Self {
            relay,
            event,
            wait_for_ok_timeout: Duration::from_secs(10),
            wait_for_authentication_timeout: Duration::from_secs(10),
        }
    }

    /// Timeout for waiting for the `OK` message from relay (default: 10 sec)
    #[inline]
    pub fn ok_timeout(mut self, timeout: Duration) -> Self {
        self.wait_for_ok_timeout = timeout;
        self
    }

    /// Timeout for waiting that relay authenticates (default: 10 sec)
    #[inline]
    pub fn authentication_timeout(mut self, timeout: Duration) -> Self {
        self.wait_for_authentication_timeout = timeout;
        self
    }

    async fn send(
        &self,
        notifications: &mut broadcast::Receiver<RelayNotification>,
        event: &Event,
    ) -> Result<(bool, String), Error> {
        // Send the EVENT message
        self.relay
            .send_msg(ClientMessage::Event(Cow::Borrowed(event)))
            .await?;

        // Wait for OK
        self.relay
            .inner
            .wait_for_ok(notifications, &event.id, self.wait_for_ok_timeout)
            .await
    }

    async fn exec(self) -> Result<EventId, Error> {
        // Health, write permission and number of messages checks are executed in `batch_msg` method.

        // Subscribe to notifications
        let mut notifications = self.relay.inner.internal_notification_sender.subscribe();

        // Send event
        let (status, message) = self.send(&mut notifications, self.event).await?;

        // Check status
        if status {
            return Ok(self.event.id);
        }

        // If auth required, wait for authentication adn resend it
        if let Some(MachineReadablePrefix::AuthRequired) = MachineReadablePrefix::parse(&message) {
            // Check if NIP42 auth is enabled and signer is set
            let has_signer: bool = self.relay.inner.state.has_signer().await;
            if self.relay.inner.state.is_auto_authentication_enabled() && has_signer {
                // Wait that relay authenticate
                wait_for_authentication(&mut notifications, self.wait_for_authentication_timeout)
                    .await?;

                // Try to resend event
                let (status, message) = self.send(&mut notifications, self.event).await?;

                // Check status
                return if status {
                    Ok(self.event.id)
                } else {
                    Err(Error::RelayMessage(message))
                };
            }
        }

        Err(Error::RelayMessage(message))
    }
}

async fn wait_for_authentication(
    notifications: &mut broadcast::Receiver<RelayNotification>,
    timeout: Duration,
) -> Result<(), Error> {
    time::timeout(Some(timeout), async {
        while let Ok(notification) = notifications.recv().await {
            match notification {
                RelayNotification::Authenticated => {
                    return Ok(());
                }
                RelayNotification::AuthenticationFailed => {
                    return Err(Error::AuthenticationFailed);
                }
                RelayNotification::RelayStatus { status } => {
                    if status.is_disconnected() {
                        return Err(Error::NotConnected);
                    }
                }
                _ => (),
            }
        }

        Err(Error::PrematureExit)
    })
    .await
    .ok_or(Error::Timeout)?
}

impl<'relay, 'event> IntoFuture for SendEvent<'relay, 'event>
where
    'event: 'relay,
{
    type Output = Result<EventId, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'relay>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl<'relay, 'event> Blocking for SendEvent<'relay, 'event> where 'event: 'relay {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nostr::prelude::*;
    use nostr_relay_builder::prelude::*;

    use super::*;

    #[tokio::test]
    async fn test_ok_msg() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let relay: Relay = Relay::new(url);

        relay
            .try_connect()
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        relay.send_event(&event).await.unwrap();
    }

    #[tokio::test]
    async fn test_nip42_send_event() {
        // Mock relay
        let opts = LocalRelayBuilderNip42 {
            mode: LocalRelayBuilderNip42Mode::Write,
        };
        let mock = LocalRelay::builder().nip42(opts).build();
        mock.run().await.unwrap();
        let url = mock.url().await;

        let relay: Relay = Relay::new(url);

        relay.inner.state.automatic_authentication(true);

        relay.connect();

        // Signer
        let keys = Keys::generate();

        // Send as unauthenticated (MUST return error)
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        let err = relay.send_event(&event).await.unwrap_err();
        if let crate::relay::Error::RelayMessage(msg) = err {
            assert_eq!(
                MachineReadablePrefix::parse(&msg).unwrap(),
                MachineReadablePrefix::AuthRequired
            );
        } else {
            panic!("Unexpected error");
        }

        // Set a signer
        relay.inner.state.set_signer(keys.clone()).await;

        // Send as authenticated
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        assert!(relay.send_event(&event).await.is_ok());
    }
}
