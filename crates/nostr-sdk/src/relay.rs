// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{bounded, select, Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use nostr_sdk_base::{ClientMessage, Event as NostrEvent, Keys, RelayMessage, SubscriptionFilter};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[cfg(feature = "blocking")]
use crate::new_current_thread;
use crate::subscription::Subscription;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayStatus {
    Disconnected,
    Connected,
    Connecting,
}

#[derive(Debug)]
enum RelayEvent {
    SendMsg(Box<ClientMessage>),
    Close,
}

#[derive(Clone)]
pub struct Relay {
    url: Url,
    //proxy: Option<SocketAddr>,
    status: Arc<Mutex<RelayStatus>>,
    pool_sender: Sender<RelayPoolEvent>,
    relay_sender: Option<Sender<RelayEvent>>,
}

impl Relay {
    pub fn new(
        url: &str,
        pool_sender: Sender<RelayPoolEvent>,
        //proxy: Option<SocketAddr>,
    ) -> Result<Self> {
        Ok(Self {
            url: Url::parse(url)?,
            //proxy,
            status: Arc::new(Mutex::new(RelayStatus::Disconnected)),
            pool_sender,
            relay_sender: None,
        })
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub async fn status(&self) -> RelayStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    pub async fn set_status(&self, status: RelayStatus) {
        let mut s = self.status.lock().await;
        *s = status;
    }

    pub async fn connect(&mut self) {
        let url: String = self.url.to_string();

        self.set_status(RelayStatus::Connecting).await;
        log::debug!("Connecting to relay {}", url);

        match tokio_tungstenite::connect_async(&self.url).await {
            Ok((stream, _)) => {
                log::debug!("Successfully connected to relay {}", url);
                self.set_status(RelayStatus::Connected).await;

                let (mut ws_tx, mut ws_rx) = stream.split();

                let (relay_sender, relay_receiver) = bounded::<RelayEvent>(32);

                self.relay_sender = Some(relay_sender);

                let relay = self.clone();
                let func_relay_event = async move {
                    log::debug!("Relay Event Thread Started");
                    loop {
                        select! {
                            recv(relay_receiver) -> result => {
                                if let Ok(relay_event) = result {
                                    match relay_event {
                                        RelayEvent::SendMsg(msg) => {
                                            log::trace!("Sending message {}", msg.to_json());
                                            if let Err(e) = ws_tx.send(Message::Text(msg.to_json())).await {
                                                log::error!("RelayEvent::SendMsg error: {:?}", e);
                                            };
                                        }
                                        RelayEvent::Close => {
                                            if let Err(e) = ws_tx.close().await {
                                                log::error!("RelayEvent::Close error: {:?}", e);
                                            };
                                            break;
                                        }
                                    }
                                }
                            },
                            default(Duration::from_secs(60)) => if let Err(e) = ws_tx.send(Message::Ping(Vec::new())).await {
                                log::error!("Ping error: {:?}", e);
                                break;
                            },
                        }
                    }

                    relay.set_status(RelayStatus::Disconnected).await;

                    log::debug!("Closed RELAY TX to WS RX {}", relay.url);
                };

                #[cfg(feature = "blocking")]
                match new_current_thread() {
                    Ok(rt) => {
                        std::thread::spawn(move || {
                            rt.block_on(async move { func_relay_event.await });
                        });
                    }
                    Err(e) => log::error!("Impossible to crate new current thread: {:?}", e),
                };

                #[cfg(not(feature = "blocking"))]
                tokio::spawn(func_relay_event);

                let relay = self.clone();
                let func_relay_msg = async move {
                    log::debug!("Relay Message Thread Started");
                    while let Some(msg_res) = ws_rx.next().await {
                        if let Ok(msg) = msg_res {
                            let data: Vec<u8> = msg.into_data();

                            match String::from_utf8(data) {
                                Ok(data) => match RelayMessage::from_json(&data) {
                                    Ok(msg) => {
                                        log::trace!("Received data: {}", &msg.to_json());
                                        if let Err(err) =
                                            relay.pool_sender.send(RelayPoolEvent::ReceivedMsg {
                                                relay_url: relay.url(),
                                                msg,
                                            })
                                        {
                                            log::error!("[CH Relay -> RelayPool] {}", &err);
                                        }
                                    }
                                    Err(err) => {
                                        log::error!("{}", err);
                                    }
                                },
                                Err(err) => log::error!("{}", err),
                            }
                        }
                    }

                    if let Err(e) = relay
                        .pool_sender
                        .send(RelayPoolEvent::RelayDisconnected(relay.url()))
                    {
                        log::error!("pool_send error: {}", e.to_string())
                    };

                    relay.disconnect().await;

                    log::error!("Closed WS RX to RELAY POOL TX {}", relay.url);
                };

                #[cfg(feature = "blocking")]
                match new_current_thread() {
                    Ok(rt) => {
                        std::thread::spawn(move || {
                            rt.block_on(async move { func_relay_msg.await });
                        });
                    }
                    Err(e) => log::error!("Impossible to crate new current thread: {:?}", e),
                };

                #[cfg(not(feature = "blocking"))]
                tokio::spawn(func_relay_msg);
            }
            Err(err) => {
                self.set_status(RelayStatus::Disconnected).await;
                log::error!("Impossible to connect to relay {}: {}", url, err);
            }
        }
    }

    pub async fn disconnect(&self) {
        self.send_relay_event(RelayEvent::Close).await;
    }

    pub async fn send_msg(&self, msg: ClientMessage) {
        self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)))
            .await;
    }

    async fn send_relay_event(&self, relay_msg: RelayEvent) {
        if let Some(relay_sender) = &self.relay_sender {
            if let Err(e) = relay_sender.send(relay_msg) {
                log::error!("send_relay_event error: {}", e.to_string())
            };
        }
    }
}

#[derive(Debug)]
pub enum RelayPoolEvent {
    RelayDisconnected(Url),
    ReceivedMsg { relay_url: Url, msg: RelayMessage },
    RemoveContactEvents(Keys),
    EventSent(NostrEvent),
}

#[derive(Debug, Clone)]
pub enum RelayPoolNotifications {
    ReceivedEvent(NostrEvent),
    RelayDisconnected(String),
}

struct RelayPoolTask {
    receiver: Receiver<RelayPoolEvent>,
    notification_sender: Sender<RelayPoolNotifications>,
    events: HashMap<String, Box<NostrEvent>>,
}

impl RelayPoolTask {
    pub fn new(
        pool_task_receiver: Receiver<RelayPoolEvent>,
        notification_sender: Sender<RelayPoolNotifications>,
    ) -> Self {
        Self {
            receiver: pool_task_receiver,
            events: HashMap::new(),
            notification_sender,
        }
    }

    pub async fn run(&mut self) {
        log::debug!("RelayPoolTask Thread Started");
        while let Ok(msg) = self.receiver.recv() {
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
                    v.pubkey != contact_keys.public_key
                        && v.tags[0].content() != contact_keys.public_key.to_string()
                });
            }
            RelayPoolEvent::RelayDisconnected(url) => {
                if let Err(e) = self
                    .notification_sender
                    .send(RelayPoolNotifications::RelayDisconnected(url.to_string()))
                {
                    log::error!("RelayPoolNotifications::RelayDisconnected error: {:?}", e);
                };
            }
        }
    }
}

pub struct RelayPool {
    relays: HashMap<String, Relay>,
    subscription: Subscription,
    pool_task_sender: Sender<RelayPoolEvent>,
    notification_receiver: Receiver<RelayPoolNotifications>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::new()
    }
}

impl RelayPool {
    pub fn new() -> Self {
        let (notification_sender, notification_receiver) = bounded(64);
        let (pool_task_sender, pool_task_receiver) = bounded(64);

        let mut relay_pool_task = RelayPoolTask::new(pool_task_receiver, notification_sender);

        #[cfg(feature = "blocking")]
        match new_current_thread() {
            Ok(rt) => {
                std::thread::spawn(move || {
                    rt.block_on(async move { relay_pool_task.run().await });
                });
            }
            Err(e) => log::error!("Impossible to crate new current thread: {:?}", e),
        };

        #[cfg(not(feature = "blocking"))]
        tokio::spawn(async move { relay_pool_task.run().await });

        Self {
            relays: HashMap::new(),
            subscription: Subscription::new(),
            pool_task_sender,
            notification_receiver,
        }
    }

    pub fn notifications(&self) -> Receiver<RelayPoolNotifications> {
        self.notification_receiver.clone()
    }

    pub fn relays(&self) -> HashMap<String, Relay> {
        self.relays.clone()
    }

    pub fn list_relays(&self) -> Vec<Relay> {
        self.relays.iter().map(|(_k, v)| v.clone()).collect()
    }

    pub fn subscription(&self) -> Subscription {
        self.subscription.clone()
    }

    pub fn add_relay(&mut self, url: &str /* proxy: Option<SocketAddr> */) -> Result<()> {
        let relay = Relay::new(url, self.pool_task_sender.clone() /* proxy */)?;
        self.relays.insert(url.into(), relay);
        Ok(())
    }

    pub async fn remove_relay(&mut self, url: &str) -> Result<()> {
        self.disconnect_relay(url).await;
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

    pub async fn send_event(&self, ev: NostrEvent) {
        //Send to pool task to save in all received events
        if !self.relays.is_empty() {
            if let Err(e) = self
                .pool_task_sender
                .send(RelayPoolEvent::EventSent(ev.clone()))
            {
                log::error!("send_ev send error: {}", e.to_string());
            };

            for (_k, v) in self.relays.iter() {
                v.send_relay_event(RelayEvent::SendMsg(Box::new(ClientMessage::new_event(
                    ev.clone(),
                ))))
                .await;
            }
        }
    }

    pub async fn start_sub(&mut self, filters: Vec<SubscriptionFilter>) {
        self.subscription.update_filters(filters.clone());
        for (k, _) in self.relays.clone().iter() {
            self.subscribe_relay(k).await;
        }
    }

    async fn subscribe_relay(&mut self, url: &str) {
        if let Some(relay) = self.relays.get(url) {
            if let RelayStatus::Connected = relay.status().await {
                let channel = self.subscription.get_channel(url);
                relay
                    .send_msg(ClientMessage::new_req(
                        channel.id.to_string(),
                        self.subscription.get_filters(),
                    ))
                    .await;
            }
        }
    }

    async fn unsubscribe_relay(&mut self, url: &str) {
        if let Some(relay) = self.relays.get(url) {
            if let RelayStatus::Connected = relay.status().await {
                if let Some(channel) = self.subscription.remove_channel(url) {
                    relay
                        .send_msg(ClientMessage::close(channel.id.to_string()))
                        .await;
                }
            }
        }
    }

    pub async fn connect_all(&mut self) {
        for (relay_url, relay) in self.relays.clone().iter() {
            if let RelayStatus::Disconnected = relay.status().await {
                self.connect_relay(relay_url).await
            }
        }
    }

    pub async fn connect_relay(&mut self, url: &str) {
        if let Some(relay) = self.relays.get_mut(&url.to_string()) {
            relay.connect().await;
            self.subscribe_relay(url).await;
        } else {
            log::error!("Impossible to connect to relay {}", url);
        }
    }

    pub async fn disconnect_relay(&mut self, url: &str) {
        if let Some(relay) = self.relays.get_mut(&url.to_string()) {
            relay.disconnect().await;
            self.unsubscribe_relay(url).await;
        } else {
            log::error!("Impossible to disconnect from relay {}", url);
        }
    }
}
