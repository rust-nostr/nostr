// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use nostr::url::Url;
use nostr::{ClientMessage, Event, RelayMessage, Sha256Hash, SubscriptionFilter};
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::time;
use uuid::Uuid;

use super::{Error as RelayError, Relay};
#[cfg(feature = "blocking")]
use crate::new_current_thread;
use crate::subscription::Subscription;

pub static SUBSCRIPTION: Lazy<Mutex<Subscription>> = Lazy::new(|| Mutex::new(Subscription::new()));

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Relay error
    #[error("relay error: {0}")]
    Relay(#[from] RelayError),
    /// No relay connected
    #[error("no relay connected")]
    NoRelayConnected,
}

#[derive(Debug)]
pub enum RelayPoolMessage {
    ReceivedMsg { relay_url: Url, msg: RelayMessage },
    EventSent(Box<Event>),
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum RelayPoolNotification {
    Event(Url, Event),
    Message(Url, RelayMessage),
    Shutdown,
}

struct RelayPoolTask {
    receiver: Receiver<RelayPoolMessage>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    events: VecDeque<Sha256Hash>,
}

const MAX_EVENTS: usize = 100000;

impl RelayPoolTask {
    pub fn new(
        pool_task_receiver: Receiver<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
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
            match msg {
                RelayPoolMessage::ReceivedMsg { relay_url, msg } => {
                    let _ = self
                        .notification_sender
                        .send(RelayPoolNotification::Message(
                            relay_url.clone(),
                            msg.clone(),
                        ));

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
                                    RelayPoolNotification::Event(relay_url, event.as_ref().clone());

                                let _ = self.notification_sender.send(notification);
                            }
                        }
                    }
                }
                RelayPoolMessage::EventSent(event) => {
                    self.add_event(event.id);
                }
                RelayPoolMessage::Shutdown => {
                    if let Err(e) = self
                        .notification_sender
                        .send(RelayPoolNotification::Shutdown)
                    {
                        log::error!("Impossible to send shutdown notification: {}", e);
                    }
                    log::debug!("Exited from RelayPoolTask thread");
                    self.receiver.close();
                    break;
                }
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

#[derive(Debug, Clone)]
pub struct RelayPool {
    relays: Arc<Mutex<HashMap<Url, Relay>>>,
    pool_task_sender: Sender<RelayPoolMessage>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::new()
    }
}

impl RelayPool {
    /// Create new `RelayPool`
    pub fn new() -> Self {
        let (notification_sender, _) = broadcast::channel(1024);
        let (pool_task_sender, pool_task_receiver) = mpsc::channel(1024);

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
            pool_task_sender,
            notification_sender,
        }
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.notification_sender.subscribe()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        let relays = self.relays.lock().await;
        relays.clone()
    }

    /// Get subscriptions
    pub async fn subscription(&self) -> Subscription {
        let subscription = SUBSCRIPTION.lock().await;
        subscription.clone()
    }

    /// Add new relay
    pub async fn add_relay(&self, url: Url, proxy: Option<SocketAddr>) {
        let mut relays = self.relays.lock().await;
        if !relays.contains_key(&url) {
            let relay = Relay::new(
                url,
                self.pool_task_sender.clone(),
                self.notification_sender.clone(),
                proxy,
            );
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

    /// Send client message
    pub async fn send_client_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        let relays = self.relays.lock().await;

        if relays.is_empty() {
            return Err(Error::NoRelayConnected);
        }

        if let ClientMessage::Event { event } = &msg {
            if let Err(err) = self
                .pool_task_sender
                .send(RelayPoolMessage::EventSent(event.clone()))
                .await
            {
                log::error!("{}", err.to_string());
            };
        }

        for relay in relays.values() {
            relay.send_msg(msg.clone()).await?;
        }

        Ok(())
    }

    /// Subscribe to filters
    pub async fn subscribe(&self, filters: Vec<SubscriptionFilter>) -> Result<(), Error> {
        let relays = self.relays.lock().await;

        {
            let mut subscription = SUBSCRIPTION.lock().await;
            subscription.update_filters(filters.clone());
        }

        for relay in relays.values() {
            relay.subscribe().await?;
        }

        Ok(())
    }

    /// Unsubscribe from filters
    pub async fn unsubscribe(&self) -> Result<(), Error> {
        let relays = self.relays.lock().await;
        for relay in relays.values() {
            relay.unsubscribe().await?;
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
            if let RelayPoolNotification::Message(_, msg) = notification {
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

    pub fn req_events_of(&self, filters: Vec<SubscriptionFilter>) {
        let this = self.clone();
        let req_events_thread = async move {
            let id = Uuid::new_v4();

            let relays = this.relays().await;

            // Subscribe
            for relay in relays.values() {
                if let Err(e) = relay
                    .send_msg(ClientMessage::new_req(id.to_string(), filters.clone()))
                    .await
                {
                    log::error!(
                        "Impossible to send REQ to {}: {}",
                        relay.url(),
                        e.to_string()
                    );
                };
            }

            let mut notifications = this.notifications();

            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Message(
                    _,
                    RelayMessage::EndOfStoredEvents { subscription_id },
                ) = notification
                {
                    if subscription_id == id.to_string() {
                        break;
                    }
                }
            }

            // Unsubscribe
            for relay in relays.values() {
                if let Err(e) = relay.send_msg(ClientMessage::close(id.to_string())).await {
                    log::error!(
                        "Impossible to close subscription with {}: {}",
                        relay.url(),
                        e.to_string()
                    );
                }
            }
        };

        #[cfg(feature = "blocking")]
        match new_current_thread() {
            Ok(rt) => {
                std::thread::spawn(move || {
                    rt.block_on(async move { req_events_thread.await });
                    rt.shutdown_timeout(Duration::from_millis(100));
                });
            }
            Err(e) => log::error!("Impossible to create new thread: {:?}", e),
        };

        #[cfg(not(feature = "blocking"))]
        tokio::task::spawn(async move { req_events_thread.await });
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self, wait_for_connection: bool) {
        let relays = self.relays.lock().await;
        for relay in relays.values() {
            self.connect_relay(relay, wait_for_connection).await;
        }
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
    pub async fn connect_relay(&self, relay: &Relay, wait_for_connection: bool) {
        relay.connect(wait_for_connection).await;
    }

    /// Disconnect from relay
    pub async fn disconnect_relay(&self, relay: &Relay) -> Result<(), Error> {
        relay.terminate().await?;
        Ok(())
    }

    /// Completly shutdown pool
    pub async fn shutdown(self) -> Result<(), Error> {
        self.disconnect().await?;
        time::sleep(Duration::from_secs(3)).await;
        if let Err(e) = self.pool_task_sender.send(RelayPoolMessage::Shutdown).await {
            log::error!("Impossible to shutdown pool: {}", e.to_string());
        };
        Ok(())
    }
}
