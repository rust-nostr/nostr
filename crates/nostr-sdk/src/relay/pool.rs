// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "blocking")]
use std::time::Duration;

use nostr::url::Url;
use nostr::{ClientMessage, Event, RelayMessage, Sha256Hash, SubscriptionFilter};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use super::{Error as RelayError, Relay};
#[cfg(feature = "blocking")]
use crate::new_current_thread;
use crate::subscription::Subscription;

#[derive(Debug)]
pub enum Error {
    /// Relay error
    Relay(RelayError),
    /// No relay connected
    NoRelayConnected,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relay(err) => write!(f, "relay error: {}", err),
            Self::NoRelayConnected => write!(f, "no relay connected"),
        }
    }
}

impl std::error::Error for Error {}

impl From<RelayError> for Error {
    fn from(err: RelayError) -> Self {
        Self::Relay(err)
    }
}

#[derive(Debug)]
pub enum RelayPoolEvent {
    ReceivedMsg { relay_url: Url, msg: RelayMessage },
    EventSent(Event),
}

#[derive(Debug, Clone)]
pub enum RelayPoolNotifications {
    ReceivedEvent(Event),
    ReceivedMessage(RelayMessage),
}

struct RelayPoolTask {
    receiver: Receiver<RelayPoolEvent>,
    notification_sender: broadcast::Sender<RelayPoolNotifications>,
    events: VecDeque<Sha256Hash>,
}

const MAX_EVENTS: usize = 100000;

impl RelayPoolTask {
    pub fn new(
        pool_task_receiver: Receiver<RelayPoolEvent>,
        notification_sender: broadcast::Sender<RelayPoolNotifications>,
    ) -> Self {
        Self {
            receiver: pool_task_receiver,
            events: VecDeque::new(),
            notification_sender,
        }
    }

    pub async fn run(&mut self) {
        log::debug!("RelayPoolTask Thread Started");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn handle_message(&mut self, msg: RelayPoolEvent) {
        match msg {
            RelayPoolEvent::ReceivedMsg { relay_url: _, msg } => {
                let _ = self
                    .notification_sender
                    .send(RelayPoolNotifications::ReceivedMessage(msg.clone()));

                if let RelayMessage::Event {
                    subscription_id: _,
                    event,
                } = msg
                {
                    //Verifies if the event is valid
                    if event.verify().is_ok() {
                        //Adds only new events
                        if !self.events.contains(&event.id) {
                            self.add_event(event.id);
                            let notification =
                                RelayPoolNotifications::ReceivedEvent(event.as_ref().clone());

                            let _ = self.notification_sender.send(notification);
                        }
                    }
                }
            }
            RelayPoolEvent::EventSent(event) => {
                self.add_event(event.id);
            }
        }
    }

    fn add_event(&mut self, event_id: Sha256Hash) {
        while self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event_id);
    }
}

#[derive(Clone)]
pub struct RelayPool {
    relays: Arc<Mutex<HashMap<Url, Relay>>>,
    subscription: Arc<Mutex<Subscription>>,
    pool_task_sender: Sender<RelayPoolEvent>,
    notification_sender: broadcast::Sender<RelayPoolNotifications>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::new()
    }
}

impl RelayPool {
    /// Create new `RelayPool`
    pub fn new() -> Self {
        let (notification_sender, _) = broadcast::channel(64);
        let (pool_task_sender, pool_task_receiver) = mpsc::channel(64);

        let mut relay_pool_task =
            RelayPoolTask::new(pool_task_receiver, notification_sender.clone());

        #[cfg(feature = "blocking")]
        match new_current_thread() {
            Ok(rt) => {
                std::thread::spawn(move || {
                    rt.block_on(async move { relay_pool_task.run().await });
                    rt.shutdown_timeout(Duration::from_millis(100));
                });
            }
            Err(e) => log::error!("Impossible to create new thread: {:?}", e),
        };

        #[cfg(not(feature = "blocking"))]
        tokio::task::spawn(async move { relay_pool_task.run().await });

        Self {
            relays: Arc::new(Mutex::new(HashMap::new())),
            subscription: Arc::new(Mutex::new(Subscription::new())),
            pool_task_sender,
            notification_sender,
        }
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotifications> {
        self.notification_sender.subscribe()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        let relays = self.relays.lock().await;
        relays.clone()
    }

    /// Get subscriptions
    pub async fn subscription(&self) -> Subscription {
        let subscription = self.subscription.lock().await;
        subscription.clone()
    }

    /// Add new relay
    pub async fn add_relay(&self, url: Url, proxy: Option<SocketAddr>) {
        let mut relays = self.relays.lock().await;
        if !relays.contains_key(&url) {
            let relay = Relay::new(url, self.pool_task_sender.clone(), proxy);
            relays.insert(relay.url(), relay);
        }
    }

    /// Disconnect and remove relay
    pub async fn remove_relay(&self, url: Url) {
        let mut relays = self.relays.lock().await;
        if let Some(relay) = relays.remove(&url) {
            if self.disconnect_relay(&relay).await.is_err() {
                relays.insert(url, relay);
            }
        }
    }

    /// Send event
    pub async fn send_event(&self, event: Event) -> Result<(), Error> {
        let relays = self.relays.lock().await;

        //Send to pool task to save in all received events
        if relays.is_empty() {
            return Err(Error::NoRelayConnected);
        }

        if let Err(err) = self
            .pool_task_sender
            .send(RelayPoolEvent::EventSent(event.clone()))
            .await
        {
            log::error!("send_event error: {}", err.to_string());
        };

        for relay in relays.values() {
            relay
                .send_msg(ClientMessage::new_event(event.clone()))
                .await?;
        }

        Ok(())
    }

    /// Subscribe to filters
    pub async fn subscribe(&self, filters: Vec<SubscriptionFilter>) -> Result<(), Error> {
        let relays = self.relays.lock().await;

        {
            let mut subscription = self.subscription.lock().await;
            subscription.update_filters(filters.clone());
        }

        for relay in relays.values() {
            self.subscribe_relay(relay).await?;
        }

        Ok(())
    }

    /// Unsubscribe from filters
    pub async fn unsubscribe(&self) -> Result<(), Error> {
        let relays = self.relays.lock().await;
        for relay in relays.values() {
            self.unsubscribe_relay(relay).await?;
        }

        Ok(())
    }

    async fn subscribe_relay(&self, relay: &Relay) -> Result<Uuid, Error> {
        let mut subscription = self.subscription.lock().await;
        let channel = subscription.get_channel(&relay.url());
        let channel_id = channel.id();

        relay
            .send_msg(ClientMessage::new_req(
                channel_id.to_string(),
                subscription.get_filters(),
            ))
            .await?;

        Ok(channel_id)
    }

    async fn unsubscribe_relay(&self, relay: &Relay) -> Result<(), Error> {
        let mut subscription = self.subscription.lock().await;
        if let Some(channel) = subscription.remove_channel(&relay.url()) {
            relay
                .send_msg(ClientMessage::close(channel.id().to_string()))
                .await?;
        }

        Ok(())
    }

    pub async fn get_events_of(
        &self,
        filters: Vec<SubscriptionFilter>,
    ) -> Result<Vec<Event>, Error> {
        let mut events: Vec<Event> = Vec::new();

        let id = Uuid::new_v4();

        let relays = self.relays.lock().await;

        // Subscribe
        for relay in relays.values() {
            relay
                .send_msg(ClientMessage::new_req(id.to_string(), filters.clone()))
                .await?;
        }

        let mut notifications = self.notifications();

        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotifications::ReceivedMessage(msg) = notification {
                match msg {
                    RelayMessage::Event {
                        subscription_id,
                        event,
                    } => {
                        if subscription_id == id.to_string() {
                            events.push(event.as_ref().clone());
                        }
                    }
                    RelayMessage::EndOfStoredEvents { subscription_id } => {
                        if subscription_id == id.to_string() {
                            break;
                        }
                    }
                    _ => (),
                };
            }
        }

        // Unsubscribe
        for relay in relays.values() {
            relay.send_msg(ClientMessage::close(id.to_string())).await?;
        }

        Ok(events)
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self, wait_for_connection: bool) -> Result<(), Error> {
        let relays = self.relays.lock().await;
        for relay in relays.values() {
            self.connect_relay(relay, wait_for_connection).await?;
        }

        Ok(())
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<(), Error> {
        let relays = self.relays.lock().await;
        for relay in relays.values() {
            self.disconnect_relay(relay).await?;
        }

        Ok(())
    }

    /// Connect to relay
    pub async fn connect_relay(
        &self,
        relay: &Relay,
        wait_for_connection: bool,
    ) -> Result<(), Error> {
        relay.connect(wait_for_connection).await;
        self.subscribe_relay(relay).await?;
        Ok(())
    }

    /// Disconnect from relay
    pub async fn disconnect_relay(&self, relay: &Relay) -> Result<(), Error> {
        relay.terminate().await?;
        self.unsubscribe_relay(relay).await?;
        Ok(())
    }
}
