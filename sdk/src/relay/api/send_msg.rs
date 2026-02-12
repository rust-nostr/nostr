use std::future::IntoFuture;
use std::time::Duration;

use nostr::ClientMessage;

use crate::future::BoxedFuture;
use crate::relay::{Error, Relay};

/// Send the client message
#[must_use = "Does nothing unless you await!"]
pub struct SendMessage<'relay, 'msg> {
    relay: &'relay Relay,
    msg: ClientMessage<'msg>,
    wait_until_sent: Option<Duration>,
}

impl<'relay, 'msg> SendMessage<'relay, 'msg> {
    pub(crate) fn new(relay: &'relay Relay, msg: ClientMessage<'msg>) -> Self {
        Self {
            relay,
            msg,
            wait_until_sent: None,
        }
    }

    #[inline]
    pub(crate) fn maybe_wait_until_sent(mut self, wait_until_sent: Option<Duration>) -> Self {
        self.wait_until_sent = wait_until_sent;
        self
    }

    /// Wait that message is sent
    #[inline]
    pub fn wait_until_sent(mut self, timeout: Duration) -> Self {
        self.wait_until_sent = Some(timeout);
        self
    }
}

impl<'relay, 'msg> IntoFuture for SendMessage<'relay, 'msg>
where
    'msg: 'relay,
{
    type Output = Result<(), Error>;
    type IntoFuture = BoxedFuture<'relay, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            self.relay
                .inner
                .send_msg(self.msg, self.wait_until_sent)
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use nostr::{Filter, RelayUrl, SubscriptionId};

    use super::*;

    // At the moment, before consider a relay disconnected, are required at least 2 attempts and a success rate < 90%
    #[tokio::test]
    async fn test_send_message_to_non_connected_relay() {
        let url = RelayUrl::parse("ws://127.0.0.1:11123").unwrap();
        let relay: Relay = Relay::new(url);

        let msg = ClientMessage::req(SubscriptionId::generate(), Filter::new().limit(10));

        // First attempt
        let res = relay.try_connect().timeout(Duration::from_secs(1)).await;
        assert!(matches!(res.unwrap_err(), Error::Transport(_)));

        // Relay is disconnected, but is required another attempt before consider it non-connected
        let res = relay.send_msg(msg.clone()).await;
        assert!(res.is_ok());

        // Second attempt
        let res = relay.try_connect().timeout(Duration::from_secs(1)).await;
        assert!(matches!(res.unwrap_err(), Error::Transport(_)));

        // Relay is disconnected, we have a 2 attempts and success rate is < 90%
        let res = relay.send_msg(msg).await;
        assert!(matches!(res.unwrap_err(), Error::NotConnected));
    }
}
