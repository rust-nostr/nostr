// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Wasm Relay Module

use std::sync::Arc;
use std::time::Duration;

use futures_util::{Future};
use ewebsock::{WsEvent, WsMessage, WsSender};
use futures::executor::block_on;
use nostr::{ClientMessage, Event, Filter, RelayMessage, SubscriptionId, Url};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use wasm_thread as thread;

pub mod pool;

use self::pool::{RelayPoolMessage, RelayPoolNotification};
use super::{RelayEvent, RelayOptions, RelayStatus};

type Message = RelayEvent;

/// WsSender wrapper
pub struct WsSend(WsSender);

impl WsSend {
    /// Send
    pub fn send(&mut self, msg: WsMessage) {
        self.0.send(msg);
    }
}

unsafe impl Send for WsSend {}
unsafe impl Sync for WsSend {}

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
    /// Read actions disabled
    #[error("read actions are disabled for this relay")]
    ReadDisabled,
    /// Write actions disabled
    #[error("write actions are disabled for this relay")]
    WriteDisabled,
    /// Filters empty
    #[error("filters empty")]
    FiltersEmpty,
}

/// Relay
#[derive(Debug, Clone)]
pub struct Relay {
    url: Url,
    status: Arc<Mutex<RelayStatus>>,
    opts: RelayOptions,
    scheduled_for_termination: Arc<Mutex<bool>>,
    pool_sender: Sender<RelayPoolMessage>,
    relay_sender: Sender<Message>,
    relay_receiver: Arc<Mutex<Receiver<Message>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
}

impl PartialEq for Relay {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Relay {
    /// New relay
    pub fn new(
        url: Url,
        pool_sender: Sender<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        opts: RelayOptions,
    ) -> Self {
        let (relay_sender, relay_receiver) = mpsc::channel::<Message>(1024);
        Self {
            url,
            status: Arc::new(Mutex::new(RelayStatus::Initialized)),
            opts,
            scheduled_for_termination: Arc::new(Mutex::new(false)), // TODO: replace with AtomicBool
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

    /// Get [`RelayStatus`]
    pub async fn status(&self) -> RelayStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    async fn set_status(&self, status: RelayStatus) {
        let mut s = self.status.lock().await;
        *s = status;
    }

    /// Get [`RelayOptions`]
    pub fn opts(&self) -> RelayOptions {
        self.opts.clone()
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
            thread::spawn(move || {
                block_on(async move {
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
                })
            });
        }
    }

    async fn try_connect(&self) {
        let url: String = self.url.to_string();

        // Set RelayStatus to `Connecting`
        self.set_status(RelayStatus::Connecting).await;
        log::debug!("Connecting to {}", url);

        match ewebsock::connect(&url) {
            Ok((ws_tx, ws_rx)) => {
                self.set_status(RelayStatus::Connected).await;
                log::info!("Connected to {}", url);

                let ws_tx = Arc::new(Mutex::new(WsSend(ws_tx)));

                let relay = self.clone();
                thread::spawn(move || {
                    block_on(async move {
                        log::debug!("Relay Event Thread Started");
                        let mut ws_tx = ws_tx.lock().await;
                        let mut rx = relay.relay_receiver.lock().await;
                        while let Some(relay_event) = rx.recv().await {
                            match relay_event {
                                RelayEvent::SendMsg(msg) => {
                                    log::debug!("Sending message {}", msg.as_json());
                                    ws_tx.send(WsMessage::Text(msg.as_json()));
                                }
                                RelayEvent::Close => {
                                    //TODO: let _ = ws_tx.close().await;
                                    relay.set_status(RelayStatus::Disconnected).await;
                                    log::info!("Disconnected from {}", url);
                                    break;
                                }
                                RelayEvent::Terminate => {
                                    // Unsubscribe from relay
                                    if let Err(e) = relay.unsubscribe().await {
                                        log::error!(
                                            "Impossible to unsubscribe from {}: {}",
                                            relay.url(),
                                            e.to_string()
                                        )
                                    }
                                    // Close stream
                                    //TODO: let _ = ws_tx.close().await;
                                    relay.set_status(RelayStatus::Terminated).await;
                                    relay.schedule_for_termination(false).await;
                                    log::info!("Completely disconnected from {}", url);
                                    break;
                                }
                            }
                        }
                    })
                });

                let ws_rx = Arc::new(Mutex::new(ws_rx));
                let relay = self.clone();
                thread::spawn(move || {
                    block_on(async move {
                        log::debug!("Relay Message Thread Started");
                        let ws_rx = ws_rx.lock().await;
                        while let Some(msg_res) = ws_rx.try_recv() {
                            if let WsEvent::Message(WsMessage::Text(data)) = msg_res {
                                match RelayMessage::from_json(&data) {
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
                                }
                            }
                        }

                        log::debug!("Exited from Message Thread of {}", relay.url);

                        if relay.status().await != RelayStatus::Terminated {
                            if let Err(err) = relay.disconnect().await {
                                log::error!("Impossible to disconnect {}: {}", relay.url, err);
                            }
                        }
                    })
                });

                // Subscribe to relay
                if self.opts.read() {
                    if let Err(e) = self.subscribe().await {
                        match e {
                            Error::FiltersEmpty => (),
                            _ => log::error!(
                                "Impossible to subscribe to {}: {}",
                                self.url(),
                                e.to_string()
                            ),
                        }
                    }
                }
            }
            Err(err) => {
                self.set_status(RelayStatus::Disconnected).await;
                log::error!("Impossible to connect to {}: {}", url, err);
            }
        };
    }

    async fn send_relay_event(&self, relay_msg: RelayEvent) -> Result<(), Error> {
        self.relay_sender
            .send(relay_msg)
            .await
            .map_err(|_| Error::ChannelTimeout)
    }

    /// Disconnect from relay and set status to 'Disconnected'
    async fn disconnect(&self) -> Result<(), Error> {
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected) && status.ne(&RelayStatus::Terminated) {
            self.send_relay_event(RelayEvent::Close).await?;
        }
        Ok(())
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub async fn terminate(&self) -> Result<(), Error> {
        self.schedule_for_termination(true).await;
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected) && status.ne(&RelayStatus::Terminated) {
            self.send_relay_event(RelayEvent::Terminate).await?;
        }
        Ok(())
    }

    /// Send msg to relay
    ///
    /// if `wait` arg is true, this method will wait for the msg to be sent
    pub async fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        if !self.opts.write() {
            if let ClientMessage::Event(_) = msg {
                return Err(Error::WriteDisabled);
            }
        }

        if !self.opts.read() {
            if let ClientMessage::Req { .. } | ClientMessage::Close(_) = msg {
                return Err(Error::ReadDisabled);
            }
        }

        self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)))
            .await
    }

    /// Subscribe
    pub async fn subscribe(&self) -> Result<SubscriptionId, Error> {
        /* if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let mut subscription = SUBSCRIPTION.lock().await;
        let filters = subscription.get_filters();

        if filters.is_empty() {
            return Err(Error::FiltersEmpty);
        }

        let channel = subscription.get_channel(&self.url());
        let channel_id = channel.id();

        self.send_msg(ClientMessage::new_req(channel_id.clone(), filters), wait)
            .await?;

        Ok(channel_id) */

        todo!()
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self) -> Result<(), Error> {
        /* if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let mut subscription = SUBSCRIPTION.lock().await;
        if let Some(channel) = subscription.remove_channel(&self.url()) {
            self.send_msg(ClientMessage::close(channel.id()), wait)
                .await?;
        } */
        Ok(())
    }

    /// Get events of filters with custom callback
    pub async fn get_events_of_with_callback<F>(
        &self,
        filters: Vec<Filter>,
        callback: impl Fn(Event) -> F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let id = SubscriptionId::generate();

        self.send_msg(ClientMessage::new_req(id.clone(), filters))
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
                            if subscription_id.eq(&id) {
                                callback(*event).await;
                            }
                        }
                        RelayMessage::EndOfStoredEvents(subscription_id) => {
                            if subscription_id.eq(&id) {
                                break;
                            }
                        }
                        _ => log::debug!("Receive unhandled message {msg:?} on get_events_of"),
                    };
                }
            }
        };

        recv.await;

        // Unsubscribe
        self.send_msg(ClientMessage::close(id)).await?;

        Ok(())
    }

    /// Get events of filters
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
    ) -> Result<Vec<Event>, Error> {
        let events: Mutex<Vec<Event>> = Mutex::new(Vec::new());
        self.get_events_of_with_callback(filters, |event| async {
            let mut events = events.lock().await;
            events.push(event);
        })
        .await?;
        Ok(events.into_inner())
    }
}
