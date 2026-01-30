use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use nostr::ClientMessage;

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

    #[inline]
    async fn exec(self) -> Result<(), Error> {
        self.relay
            .inner
            .send_msg(self.msg, self.wait_until_sent)
            .await
    }
}

impl<'relay, 'msg> IntoFuture for SendMessage<'relay, 'msg>
where
    'msg: 'relay,
{
    type Output = Result<(), Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'relay>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl_blocking!(for<'relay, 'msg> SendMessage<'relay, 'msg> where 'msg: 'relay);
