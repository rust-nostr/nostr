// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use nostr::{ClientMessage, Event, RelayMessage, SubscriptionFilter, SubscriptionId, Url};
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::sync::Mutex;

mod net;
pub mod pool;

use self::net::Message as WsMessage;
use self::pool::RelayPoolMessage;
use self::pool::SUBSCRIPTION;
use crate::thread;
use crate::RelayPoolNotification;

type Message = (RelayEvent, Option<oneshot::Sender<bool>>);

/// [`Relay`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Channel timeout
    #[error("channel timeout")]
    ChannelTimeout,
    /// Message response timeout
    #[error("recv message response timeout")]
    RecvTimeout,
    /// Generic timeout
    #[error("timeout")]
    Timeout,
    /// Message not sent
    #[error("message not sent")]
    MessagetNotSent,
    /// Impossible to receive oneshot message
    #[error("impossible to recv msg")]
    OneShotRecvError,
}

/// Relay connection status
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Relay connected
    Connected,
    /// Connecting
    Connecting,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Relay completly disconnected
    Terminated,
}

impl fmt::Display for RelayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized => write!(f, "Initialized"),
            Self::Connected => write!(f, "Connected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

/// Relay event
#[derive(Debug)]
pub enum RelayEvent {
    /// Send [`ClientMessage`]
    SendMsg(Box<ClientMessage>),
    // Ping,
    /// Close
    Close,
    /// Completly disconnect
    Terminate,
}

/// Relay
#[derive(Debug, Clone)]
pub struct Relay {
    url: Url,
    proxy: Option<SocketAddr>,
    status: Arc<Mutex<RelayStatus>>,
    scheduled_for_termination: Arc<Mutex<bool>>,
    pool_sender: Sender<RelayPoolMessage>,
    relay_sender: Sender<Message>,
    relay_receiver: Arc<Mutex<Receiver<Message>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
}

impl Relay {
    /// Create new `Relay`
    pub fn new(
        url: Url,
        pool_sender: Sender<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        proxy: Option<SocketAddr>,
    ) -> Self {
        let (relay_sender, relay_receiver) = mpsc::channel::<Message>(1024);

        Self {
            url,
            proxy,
            status: Arc::new(Mutex::new(RelayStatus::Initialized)),
            scheduled_for_termination: Arc::new(Mutex::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
        }
    }

    /// Get relay url
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    /// Get proxy
    pub fn proxy(&self) -> Option<SocketAddr> {
        self.proxy
    }

    /// Get [`RelayStatus`]
    pub async fn status(&self) -> RelayStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    async fn set_status(&self, status: RelayStatus) {
        let mut s = self.status.lock().await;
        *s = status;
    }

    async fn is_scheduled_for_termination(&self) -> bool {
        let value = self.scheduled_for_termination.lock().await;
        *value
    }

    async fn schedule_for_termination(&self, value: bool) {
        let mut s = self.scheduled_for_termination.lock().await;
        *s = value;
    }

    /// Connect to relay and keep alive connection
    pub async fn connect(&self, wait_for_connection: bool) {
        if let RelayStatus::Initialized | RelayStatus::Terminated = self.status().await {
            if wait_for_connection {
                self.try_connect().await
            } else {
                // Update relay status
                self.set_status(RelayStatus::Disconnected).await;
            }

            let relay = self.clone();
            thread::spawn(async move {
                loop {
                    log::debug!(
                        "{} channel capacity: {}",
                        relay.url(),
                        relay.relay_sender.capacity()
                    );

                    // Schedule relay for termination
                    // Needed to terminate the auto reconnect loop, also if the relay is not connected yet.
                    if relay.is_scheduled_for_termination().await {
                        relay.set_status(RelayStatus::Terminated).await;
                        relay.schedule_for_termination(false).await;
                        log::debug!("Auto connect loop terminated for {}", relay.url);
                        break;
                    }

                    // Check status
                    match relay.status().await {
                        RelayStatus::Disconnected => relay.try_connect().await,
                        RelayStatus::Terminated => {
                            log::debug!("Auto connect loop terminated for {}", relay.url);
                            break;
                        }
                        _ => (),
                    };

                    tokio::time::sleep(Duration::from_secs(20)).await;
                }
            });
        }
    }

    async fn try_connect(&self) {
        let url: String = self.url.to_string();

        self.set_status(RelayStatus::Connecting).await;
        log::debug!("Connecting to {}", url);

        match net::get_connection(&self.url, self.proxy, None).await {
            Ok((mut ws_tx, mut ws_rx)) => {
                self.set_status(RelayStatus::Connected).await;
                log::info!("Connected to {}", url);

                let relay = self.clone();
                thread::spawn(async move {
                    log::debug!("Relay Event Thread Started");
                    let mut rx = relay.relay_receiver.lock().await;
                    while let Some((relay_event, oneshot_sender)) = rx.recv().await {
                        match relay_event {
                            RelayEvent::SendMsg(msg) => {
                                log::trace!("Sending message {}", msg.as_json());
                                if let Err(e) = ws_tx.send(WsMessage::Text(msg.as_json())).await {
                                    log::error!(
                                        "Impossible to send msg to {}: {}",
                                        relay.url(),
                                        e.to_string()
                                    );
                                    if let Some(sender) = oneshot_sender {
                                        if let Err(e) = sender.send(false) {
                                            log::error!("Impossible to send oneshot msg: {}", e);
                                        }
                                    }
                                    break;
                                };
                                if let Some(sender) = oneshot_sender {
                                    if let Err(e) = sender.send(true) {
                                        log::error!("Impossible to send oneshot msg: {}", e);
                                    }
                                }
                            }
                            /* RelayEvent::Ping => {
                                if let Err(e) = ws_tx.feed(Message::Ping(Vec::new())).await {
                                    log::error!("Ping error: {}", e);
                                    break;
                                }
                            } */
                            RelayEvent::Close => {
                                let _ = ws_tx.close().await;
                                relay.set_status(RelayStatus::Disconnected).await;
                                log::info!("Disconnected from {}", url);
                                break;
                            }
                            RelayEvent::Terminate => {
                                // Unsubscribe from relay
                                if let Err(e) = relay.unsubscribe(false).await {
                                    log::error!(
                                        "Impossible to unsubscribe from {}: {}",
                                        relay.url(),
                                        e.to_string()
                                    )
                                }
                                // Close stream
                                let _ = ws_tx.close().await;
                                relay.set_status(RelayStatus::Terminated).await;
                                relay.schedule_for_termination(false).await;
                                log::info!("Completely disconnected from {}", url);
                                break;
                            }
                        }
                    }
                });

                let relay = self.clone();
                thread::spawn(async move {
                    log::debug!("Relay Message Thread Started");
                    while let Some(msg_res) = ws_rx.next().await {
                        if let Ok(msg) = msg_res {
                            let data: Vec<u8> = msg.into_data();

                            match String::from_utf8(data) {
                                Ok(data) => match RelayMessage::from_json(&data) {
                                    Ok(msg) => {
                                        log::trace!("Received message to {}: {:?}", relay.url, msg);
                                        if let Err(err) = relay
                                            .pool_sender
                                            .send(RelayPoolMessage::ReceivedMsg {
                                                relay_url: relay.url(),
                                                msg,
                                            })
                                            .await
                                        {
                                            log::error!(
                                                "Impossible to send ReceivedMsg to pool: {}",
                                                &err
                                            );
                                        };
                                    }
                                    Err(err) => {
                                        log::error!("{}: {}", err, data);
                                    }
                                },
                                Err(err) => log::error!("{}", err),
                            }
                        }
                    }

                    log::debug!("Exited from Message Thread of {}", relay.url);

                    if relay.status().await != RelayStatus::Terminated {
                        if let Err(err) = relay.disconnect().await {
                            log::error!("Impossible to disconnect {}: {}", relay.url, err);
                        }
                    }
                });

                // Ping thread
                /* let relay = self.clone();
                thread::spawn(async move {
                    log::debug!("Relay Ping Thread Started");

                    loop {
                        tokio::time::sleep(Duration::from_secs(120)).await;
                        match relay.status().await {
                            RelayStatus::Terminated => break,
                            RelayStatus::Connected => match relay.ping().await {
                                Ok(_) => log::debug!("Ping {}", relay.url),
                                Err(err) => {
                                    log::error!("Impossible to ping {}: {}", relay.url, err);
                                    break;
                                }
                            },
                            _ => (),
                        }
                    }

                    log::debug!("Exited from Ping Thread of {}", relay.url);

                    if relay.status().await != RelayStatus::Terminated {
                        if let Err(err) = relay.disconnect().await {
                            log::error!("Impossible to disconnect {}: {}", relay.url, err);
                        }
                    }
                }); */

                // Subscribe to relay
                if let Err(e) = self.subscribe(false).await {
                    log::error!(
                        "Impossible to subscribe to {}: {}",
                        self.url(),
                        e.to_string()
                    )
                }
            }
            Err(err) => {
                self.set_status(RelayStatus::Disconnected).await;
                log::error!("Impossible to connect to {}: {}", url, err);
            }
        };
    }

    async fn send_relay_event(
        &self,
        relay_msg: RelayEvent,
        sender: Option<oneshot::Sender<bool>>,
    ) -> Result<(), Error> {
        self.relay_sender
            .send_timeout((relay_msg, sender), Duration::from_secs(60))
            .await
            .map_err(|_| Error::ChannelTimeout)
    }

    /// Ping relay
    /* async fn ping(&self) -> Result<(), Error> {
        self.send_relay_event(RelayEvent::Ping).await
    } */

    /// Disconnect from relay and set status to 'Disconnected'
    async fn disconnect(&self) -> Result<(), Error> {
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected) && status.ne(&RelayStatus::Terminated) {
            self.send_relay_event(RelayEvent::Close, None).await?;
        }
        Ok(())
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub async fn terminate(&self) -> Result<(), Error> {
        self.schedule_for_termination(true).await;
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected) && status.ne(&RelayStatus::Terminated) {
            self.send_relay_event(RelayEvent::Terminate, None).await?;
        }
        Ok(())
    }

    /// Send msg to relay
    ///
    /// if `wait` arg is true, this method will wait for the msg to be sent
    pub async fn send_msg(&self, msg: ClientMessage, wait: bool) -> Result<(), Error> {
        if wait {
            let (tx, rx) = oneshot::channel::<bool>();
            self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)), Some(tx))
                .await?;
            match tokio::time::timeout(Duration::from_secs(60), rx).await {
                Ok(result) => match result {
                    Ok(val) => {
                        if val {
                            Ok(())
                        } else {
                            Err(Error::MessagetNotSent)
                        }
                    }
                    Err(_) => Err(Error::OneShotRecvError),
                },
                Err(_) => Err(Error::RecvTimeout),
            }
        } else {
            self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)), None)
                .await
        }
    }

    /// Subscribe
    pub async fn subscribe(&self, wait: bool) -> Result<SubscriptionId, Error> {
        let mut subscription = SUBSCRIPTION.lock().await;
        let channel = subscription.get_channel(&self.url());
        let channel_id = channel.id();
        self.send_msg(
            ClientMessage::new_req(channel_id.clone(), subscription.get_filters()),
            wait,
        )
        .await?;
        Ok(channel_id)
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, wait: bool) -> Result<(), Error> {
        let mut subscription = SUBSCRIPTION.lock().await;
        if let Some(channel) = subscription.remove_channel(&self.url()) {
            self.send_msg(ClientMessage::close(channel.id()), wait)
                .await?;
        }
        Ok(())
    }

    /// Get events of filters
    pub async fn get_events_of(
        &self,
        filters: Vec<SubscriptionFilter>,
        timeout: Duration,
    ) -> Result<Vec<Event>, Error> {
        let mut events: Vec<Event> = Vec::new();

        let id = SubscriptionId::generate();

        self.send_msg(ClientMessage::new_req(id.clone(), filters.clone()), false)
            .await?;

        let mut notifications = self.notification_sender.subscribe();
        let recv = async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Message(_, msg) = notification {
                    match msg {
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        } => {
                            if subscription_id == id {
                                events.push(event.as_ref().clone());
                            }
                        }
                        RelayMessage::EndOfStoredEvents(subscription_id) => {
                            if subscription_id == id {
                                break;
                            }
                        }
                        _ => (),
                    };
                }
            }
        };

        if tokio::time::timeout(timeout, recv).await.is_err() {
            return Err(Error::Timeout);
        }

        // Unsubscribe
        self.send_msg(ClientMessage::close(id), false).await?;

        Ok(events)
    }

    /// Request events of filter. All events will be sent to notification listener
    pub fn req_events_of(&self, filters: Vec<SubscriptionFilter>, timeout: Duration) {
        let relay = self.clone();
        thread::spawn(async move {
            let id = SubscriptionId::generate();

            // Subscribe
            if let Err(e) = relay
                .send_msg(ClientMessage::new_req(id.clone(), filters.clone()), false)
                .await
            {
                log::error!(
                    "Impossible to send REQ to {}: {}",
                    relay.url(),
                    e.to_string()
                );
            };

            let mut notifications = relay.notification_sender.subscribe();
            let recv = async {
                while let Ok(notification) = notifications.recv().await {
                    if let RelayPoolNotification::Message(
                        _,
                        RelayMessage::EndOfStoredEvents(subscription_id),
                    ) = notification
                    {
                        if subscription_id == id {
                            break;
                        }
                    }
                }
            };

            if let Err(e) = tokio::time::timeout(timeout, recv).await {
                log::error!("{e}");
            }

            // Unsubscribe
            if let Err(e) = relay.send_msg(ClientMessage::close(id), false).await {
                log::error!(
                    "Impossible to close subscription with {}: {}",
                    relay.url(),
                    e.to_string()
                );
            }
        });
    }
}
