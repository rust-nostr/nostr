// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(feature = "blocking")]
use std::time::Duration;

use anyhow::{anyhow, Result};
use nostr_sdk_base::{ClientMessage, Event as NostrEvent, Keys, RelayMessage, SubscriptionFilter};
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use url::Url;
use uuid::Uuid;

use super::Relay;
#[cfg(feature = "blocking")]
use crate::new_current_thread;
use crate::subscription::Subscription;

#[derive(Debug)]
pub enum RelayPoolEvent {
    ReceivedMsg { relay_url: Url, msg: RelayMessage },
    RemoveContactEvents(Keys),
    EventSent(NostrEvent),
}

#[derive(Debug, Clone)]
pub enum RelayPoolNotifications {
    ReceivedEvent(NostrEvent),
}

struct RelayPoolTask {
    receiver: Receiver<RelayPoolEvent>,
    notification_sender: broadcast::Sender<RelayPoolNotifications>,
    events: HashMap<String, Box<NostrEvent>>,
}

impl RelayPoolTask {
    pub fn new(
        pool_task_receiver: Receiver<RelayPoolEvent>,
        notification_sender: broadcast::Sender<RelayPoolNotifications>,
    ) -> Self {
        Self {
            receiver: pool_task_receiver,
            events: HashMap::new(),
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
            RelayPoolEvent::ReceivedMsg { relay_url, msg } => {
                log::debug!("Received message from {}: {:?}", &relay_url, &msg);

                if let RelayMessage::Event {
                    event,
                    subscription_id: _,
                } = msg
                {
                    //Verifies if the event is valid
                    if event.verify().is_ok() {
                        //Adds only new events
                        if self
                            .events
                            .insert(event.id.to_string(), event.clone())
                            .is_none()
                        {
                            let notification =
                                RelayPoolNotifications::ReceivedEvent(event.as_ref().clone());

                            if let Err(e) = self.notification_sender.send(notification) {
                                log::error!("RelayPoolNotifications::ReceivedEvent error: {:?}", e);
                            };
                        }
                    }
                }
            }
            RelayPoolEvent::EventSent(ev) => {
                self.events.insert(ev.id.to_string(), Box::new(ev));
            }
            RelayPoolEvent::RemoveContactEvents(contact_keys) => {
                self.events.retain(|_, v| {
                    v.pubkey != contact_keys.public_key()
                        && v.tags[0].content() != contact_keys.public_key().to_string()
                });
            }
        }
    }
}

pub struct RelayPool {
    relays: HashMap<String, Relay>,
    subscription: Subscription,
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
            relays: HashMap::new(),
            subscription: Subscription::new(),
            pool_task_sender,
            notification_sender,
        }
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotifications> {
        self.notification_sender.subscribe()
    }

    /// Get relays
    pub fn relays(&self) -> HashMap<String, Relay> {
        self.relays.clone()
    }

    /// Get list of relays
    pub fn list_relays(&self) -> Vec<Relay> {
        self.relays.iter().map(|(_k, v)| v.clone()).collect()
    }

    /// Get subscriptions
    pub async fn subscription(&self) -> Subscription {
        self.subscription.clone()
    }

    /// Add new relay
    pub fn add_relay(&mut self, url: &str, proxy: Option<SocketAddr>) -> Result<()> {
        let relay = Relay::new(url, self.pool_task_sender.clone(), proxy)?;
        self.relays.insert(url.into(), relay);
        Ok(())
    }

    /// Disconnect and remove relay
    pub async fn remove_relay(&mut self, url: &str) -> Result<()> {
        if let Some(relay) = self.relays.remove(url) {
            if self.disconnect_relay(&relay).await.is_err() {
                self.relays.insert(url.into(), relay);
            }
        }

        Ok(())
    }

    /// Send event
    pub async fn send_event(&self, ev: NostrEvent) -> Result<()> {
        //Send to pool task to save in all received events
        if self.relays.is_empty() {
            return Err(anyhow!("No relay connected"));
        }

        if let Err(e) = self
            .pool_task_sender
            .send(RelayPoolEvent::EventSent(ev.clone()))
            .await
        {
            log::error!("send_event error: {}", e.to_string());
        };

        for (_, relay) in self.relays.iter() {
            relay.send_msg(ClientMessage::new_event(ev.clone())).await?;
        }

        Ok(())
    }

    /// Subscribe to filters
    pub async fn subscribe(&mut self, filters: Vec<SubscriptionFilter>) -> Result<()> {
        self.subscription.update_filters(filters.clone());
        for relay in self.relays.clone().values() {
            self.subscribe_relay(relay).await?;
        }

        Ok(())
    }

    /// Unsubscribe from filters
    pub async fn unsubscribe(&mut self) -> Result<()> {
        for relay in self.relays.clone().values() {
            self.unsubscribe_relay(relay).await?;
        }

        Ok(())
    }

    async fn subscribe_relay(&mut self, relay: &Relay) -> Result<Uuid> {
        let channel = self.subscription.get_channel(&relay.url());
        let channel_id = channel.id();

        relay
            .send_msg(ClientMessage::new_req(
                channel_id.to_string(),
                self.subscription.get_filters(),
            ))
            .await?;

        Ok(channel_id)
    }

    async fn unsubscribe_relay(&mut self, relay: &Relay) -> Result<()> {
        if let Some(channel) = self.subscription.remove_channel(&relay.url()) {
            relay
                .send_msg(ClientMessage::close(channel.id().to_string()))
                .await?;
        }

        Ok(())
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&mut self) -> Result<()> {
        for relay in self.relays.clone().values() {
            self.connect_relay(relay).await?;
        }

        Ok(())
    }

    /// Disconnect from all relays
    pub async fn disconnect(&mut self) -> Result<()> {
        for relay in self.relays.clone().values() {
            self.disconnect_relay(relay).await?;
        }

        Ok(())
    }

    /// Connect to relay
    pub async fn connect_relay(&mut self, relay: &Relay) -> Result<()> {
        relay.connect().await;
        self.subscribe_relay(relay).await?;
        Ok(())
    }

    /// Disconnect from relay
    pub async fn disconnect_relay(&mut self, relay: &Relay) -> Result<()> {
        relay.terminate().await?;
        self.unsubscribe_relay(relay).await?;
        Ok(())
    }
}
