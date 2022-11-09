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

use super::Relay;
use crate::subscription::Subscription;
#[cfg(feature = "blocking")]
use crate::new_current_thread;

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

    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotifications> {
        self.notification_sender.subscribe()
    }

    pub fn relays(&self) -> HashMap<String, Relay> {
        self.relays.clone()
    }

    pub fn list_relays(&self) -> Vec<Relay> {
        self.relays.iter().map(|(_k, v)| v.clone()).collect()
    }

    pub async fn subscription(&self) -> Subscription {
        self.subscription.clone()
    }

    pub fn add_relay(&mut self, url: &str, proxy: Option<SocketAddr>) -> Result<()> {
        let relay = Relay::new(url, self.pool_task_sender.clone(), proxy)?;
        self.relays.insert(url.into(), relay);
        Ok(())
    }

    pub async fn remove_relay(&mut self, url: &str) -> Result<()> {
        self.disconnect_relay(url).await?;
        self.relays.remove(url);
        Ok(())
    }

    /* pub async fn remove_contact_events(&self, contact: Contact) {
        //TODO: Remove this convertion when change contact pk to Keys type
        let c_keys = Keys::new_pub_only(&contact.pk.to_string()).unwrap();
        if let Err(e) = self
            .pool_task_sender
            .send(RelayPoolEvent::RemoveContactEvents(c_keys))
        {
            log::error!("remove_contact_events send error: {}", e.to_string())
        };
    } */

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

    pub async fn subscribe(&mut self, filters: Vec<SubscriptionFilter>) -> Result<()> {
        self.subscription.update_filters(filters.clone());
        for (k, _) in self.relays.clone().iter() {
            self.subscribe_relay(k).await?;
        }

        Ok(())
    }

    async fn subscribe_relay(&mut self, url: &str) -> Result<()> {
        if let Some(relay) = self.relays.get(url) {
            let channel = self.subscription.get_channel(url);
            relay
                .send_msg(ClientMessage::new_req(
                    channel.id.to_string(),
                    self.subscription.get_filters(),
                ))
                .await?;
        }

        Ok(())
    }

    async fn unsubscribe_relay(&mut self, url: &str) -> Result<()> {
        if let Some(relay) = self.relays.get(url) {
            if let Some(channel) = self.subscription.remove_channel(url) {
                relay
                    .send_msg(ClientMessage::close(channel.id.to_string()))
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn connect(&mut self) -> Result<()> {
        for url in self.relays.clone().keys() {
            self.connect_relay(url).await?;
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        for url in self.relays.clone().keys() {
            self.disconnect_relay(url).await?;
        }

        Ok(())
    }

    pub async fn connect_relay(&mut self, url: &str) -> Result<()> {
        if let Some(relay) = self.relays.get(&url.to_string()) {
            relay.connect().await;
            self.subscribe_relay(url).await?;
        } else {
            log::error!("Impossible to connect to {}", url);
        }

        Ok(())
    }

    pub async fn disconnect_relay(&mut self, url: &str) -> Result<()> {
        if let Some(relay) = self.relays.get(&url.to_string()) {
            relay.terminate().await?;
            self.unsubscribe_relay(url).await?;
        } else {
            log::error!("Impossible to disconnect from {}", url);
        }

        Ok(())
    }
}
