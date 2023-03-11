// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Wasm Relay Pool

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use futures::executor::block_on;
use nostr::url::Url;
use nostr::{ClientMessage, Event, EventId, Filter, RelayMessage};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use wasm_thread as thread;

use super::{Error as RelayError, Relay, RelayOptions};
use crate::subscription::Subscription;

/// [`RelayPool`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Relay error
    #[error(transparent)]
    Relay(#[from] RelayError),
    /// No relay connected
    #[error("no relay connected")]
    NoRelayConnected,
    /// Relay not found
    #[error("relay not found")]
    RelayNotFound,
}

/// Relay Pool Message
#[derive(Debug)]
pub enum RelayPoolMessage {
    /// Received new message
    ReceivedMsg {
        /// Relay url
        relay_url: Url,
        /// Relay message
        msg: RelayMessage,
    },
    /// Event sent
    EventSent(Box<Event>),
    /// Shutdown
    Shutdown,
}

/// Relay Pool Notification
#[derive(Debug, Clone)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]
    Event(Url, Event),
    /// Received a [`RelayMessage`]
    Message(Url, RelayMessage),
    /// Shutdown
    Shutdown,
}

struct RelayPoolTask {
    receiver: Receiver<RelayPoolMessage>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    events: VecDeque<EventId>,
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

                    if let RelayMessage::Event { event, .. } = msg {
                        // Verifies if the event is valid
                        if event.verify().is_ok() {
                            // Adds only new events
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

    fn add_event(&mut self, event_id: EventId) {
        while self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event_id);
    }
}

/// Relay Pool
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

        thread::spawn(move || block_on(async move { relay_pool_task.run().await }));

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
        /* let subscription = SUBSCRIPTION.lock().await;
        subscription.clone() */
        todo!()
    }

    /// Add new relay
    pub async fn add_relay(&self, url: Url, opts: RelayOptions) -> Result<(), Error> {
        let mut relays = self.relays.lock().await;
        if !relays.contains_key(&url) {
            let relay = Relay::new(
                url,
                self.pool_task_sender.clone(),
                self.notification_sender.clone(),
                opts,
            );
            relays.insert(relay.url(), relay);
        }
        Ok(())
    }

    /// Disconnect and remove relay
    pub async fn remove_relay(&self, url: Url) -> Result<(), Error> {
        let mut relays = self.relays.lock().await;
        if let Some(relay) = relays.remove(&url) {
            self.disconnect_relay(&relay).await?;
        }
        Ok(())
    }

    /// Send client message
    pub async fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        let relays = self.relays().await;

        if relays.is_empty() {
            return Err(Error::NoRelayConnected);
        }

        if let ClientMessage::Event(event) = &msg {
            if let Err(e) = self
                .pool_task_sender
                .send(RelayPoolMessage::EventSent(event.clone()))
                .await
            {
                log::error!("{e}");
            };
        }

        for (url, relay) in relays.into_iter() {
            if let Err(e) = relay.send_msg(msg.clone()).await {
                log::error!("Impossible to send msg to {url}: {e}");
            }
        }

        Ok(())
    }

    /// Send client message
    pub async fn send_msg_to(&self, url: Url, msg: ClientMessage) -> Result<(), Error> {
        let relays = self.relays().await;
        if let Some(relay) = relays.get(&url) {
            relay.send_msg(msg).await?;
            Ok(())
        } else {
            Err(Error::RelayNotFound)
        }
    }

    /// Subscribe to filters
    pub async fn subscribe(&self, filters: Vec<Filter>) {
        let relays = self.relays().await;

        /* {
            let mut subscription = SUBSCRIPTION.lock().await;
            subscription.update_filters(filters.clone());
        } */

        for relay in relays.values() {
            if let Err(e) = relay.subscribe().await {
                log::error!("{e}");
            }
        }
    }

    /// Unsubscribe from filters
    pub async fn unsubscribe(&self) {
        let relays = self.relays().await;
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe().await {
                log::error!("{e}");
            }
        }
    }

    /// Get events of filters
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
    ) -> Result<Vec<Event>, Error> {
        let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        let relays = self.relays().await;
        for (url, relay) in relays.into_iter() {
            let filters = filters.clone();
            let events = events.clone();
            let handle = thread::spawn(move|| block_on(async move {
                if let Err(e) = relay
                    .get_events_of_with_callback(filters, |event| async {
                        events.lock().await.push(event);
                    })
                    .await
                {
                    log::error!("Failed to get events from {url}: {e}");
                }
            }));
            handles.push(handle);
        }

        for handle in handles.into_iter() {
            handle.join_async().await.unwrap();
        }

        Ok(events.lock_owned().await.clone())
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self) {
        let relays = self.relays().await;
        for relay in relays.values() {
            self.connect_relay(relay).await;
        }
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.values() {
            self.disconnect_relay(relay).await?;
        }

        Ok(())
    }

    /// Connect to relay
    pub async fn connect_relay(&self, relay: &Relay) {
        relay.connect(false).await;
    }

    /// Disconnect from relay
    pub async fn disconnect_relay(&self, relay: &Relay) -> Result<(), Error> {
        relay.terminate().await?;
        Ok(())
    }

    /// Completly shutdown pool
    pub async fn shutdown(self) -> Result<(), Error> {
        self.disconnect().await?;
        if let Err(e) = self.pool_task_sender.send(RelayPoolMessage::Shutdown).await {
            log::error!("Impossible to shutdown pool: {e}");
        };
        Ok(())
    }
}
