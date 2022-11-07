// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use nostr_sdk_base::{ClientMessage, Event as NostrEvent, Keys, RelayMessage, SubscriptionFilter};
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

mod network;
mod socks;

#[cfg(feature = "blocking")]
use crate::new_current_thread;
use crate::subscription::Subscription;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayStatus {
    Initialized,
    Connected,
    Connecting,
    Disconnected,
    Terminated,
}

#[derive(Debug)]
enum RelayEvent {
    SendMsg(Box<ClientMessage>),
    Ping,
    Close,
    Terminate,
}

#[derive(Clone)]
pub struct Relay {
    url: Url,
    proxy: Option<SocketAddr>,
    status: Arc<Mutex<RelayStatus>>,
    pool_sender: Sender<RelayPoolEvent>,
    relay_sender: Sender<RelayEvent>,
    relay_receiver: Arc<Mutex<Receiver<RelayEvent>>>,
}

impl Relay {
    pub fn new(
        url: &str,
        pool_sender: Sender<RelayPoolEvent>,
        proxy: Option<SocketAddr>,
    ) -> Result<Self> {
        let (relay_sender, relay_receiver) = mpsc::channel::<RelayEvent>(64);

        Ok(Self {
            url: Url::parse(url)?,
            proxy,
            status: Arc::new(Mutex::new(RelayStatus::Initialized)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
        })
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    async fn status(&self) -> RelayStatus {
        let status = self.status.lock().await;
        status.clone()
    }

    pub async fn set_status(&self, status: RelayStatus) {
        let mut s = self.status.lock().await;
        *s = status;
    }

    pub async fn connect(&self) {
        if let RelayStatus::Initialized | RelayStatus::Terminated = self.status().await {
            // Update relay status
            self.set_status(RelayStatus::Disconnected).await;

            let relay = self.clone();
            let connection_thread = async move {
                loop {
                    // Check status
                    match relay.status().await {
                        RelayStatus::Disconnected => relay.try_connect().await,
                        RelayStatus::Terminated => break,
                        _ => (),
                    };

                    // TODO: if disconnected and connected again, get subscription filters from store (sled or something else) and send it again

                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            };

            #[cfg(feature = "blocking")]
            match new_current_thread() {
                Ok(rt) => {
                    std::thread::spawn(move || {
                        rt.block_on(async move { connection_thread.await });
                        rt.shutdown_timeout(Duration::from_millis(100));
                    });
                }
                Err(e) => log::error!("Impossible to create new thread: {:?}", e),
            };

            #[cfg(not(feature = "blocking"))]
            tokio::task::spawn(connection_thread);
        }
    }

    async fn try_connect(&self) {
        let url: String = self.url.to_string();

        self.set_status(RelayStatus::Connecting).await;
        log::debug!("Connecting to {}", url);

        match network::get_connection(&self.url, self.proxy).await {
            Ok((mut ws_tx, mut ws_rx)) => {
                self.set_status(RelayStatus::Connected).await;
                log::info!("Connected to {}", url);

                let relay = self.clone();
                let func_relay_event = async move {
                    log::debug!("Relay Event Thread Started");
                    while let Some(relay_event) = relay.relay_receiver.lock().await.recv().await {
                        match relay_event {
                            RelayEvent::SendMsg(msg) => {
                                log::trace!("Sending message {}", msg.to_json());
                                if let Err(e) = ws_tx.send(Message::Text(msg.to_json())).await {
                                    log::error!("RelayEvent::SendMsg error: {:?}", e);
                                };
                            }
                            RelayEvent::Ping => {
                                if let Err(e) = ws_tx.send(Message::Ping(Vec::new())).await {
                                    log::error!("Ping error: {:?}", e);
                                    break;
                                }
                            }
                            RelayEvent::Close => {
                                if let Err(e) = ws_tx.close().await {
                                    log::error!("RelayEvent::Close error: {:?}", e);
                                };
                                relay.set_status(RelayStatus::Disconnected).await;
                                log::info!("Disconnected from {}", url);
                                break;
                            }
                            RelayEvent::Terminate => {
                                if let Err(e) = ws_tx.close().await {
                                    log::error!("RelayEvent::Close error: {:?}", e);
                                };
                                relay.set_status(RelayStatus::Terminated).await;
                                log::info!("Completely disconnected from {}", url);
                                break;
                            }
                        }
                    }
                };

                #[cfg(feature = "blocking")]
                match new_current_thread() {
                    Ok(rt) => {
                        std::thread::spawn(move || {
                            rt.block_on(async move { func_relay_event.await });
                            rt.shutdown_timeout(Duration::from_millis(100));
                        });
                    }
                    Err(e) => log::error!("Impossible to create new thread: {:?}", e),
                };

                #[cfg(not(feature = "blocking"))]
                tokio::task::spawn(func_relay_event);

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
                                        if let Err(err) = relay
                                            .pool_sender
                                            .send(RelayPoolEvent::ReceivedMsg {
                                                relay_url: relay.url(),
                                                msg,
                                            })
                                            .await
                                        {
                                            log::error!(
                                                "Impossible to send ReceivedMsg to pool: {}",
                                                &err
                                            );
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
                        .await
                    {
                        log::error!(
                            "Impossible to send RelayDisconnected to pool: {}",
                            e.to_string()
                        )
                    };

                    if relay.status().await != RelayStatus::Terminated {
                        if let Err(err) = relay.disconnect().await {
                            log::error!("Impossible to disconnect {}: {}", relay.url, err);
                        }
                    }
                };

                #[cfg(feature = "blocking")]
                match new_current_thread() {
                    Ok(rt) => {
                        std::thread::spawn(move || {
                            rt.block_on(async move { func_relay_msg.await });
                            rt.shutdown_timeout(Duration::from_millis(100));
                        });
                    }
                    Err(e) => log::error!("Impossible to create new thread: {:?}", e),
                };

                #[cfg(not(feature = "blocking"))]
                tokio::task::spawn(func_relay_msg);

                // Ping thread
                let relay = self.clone();
                let func_relay_ping = async move {
                    log::debug!("Relay Ping Thread Started");

                    loop {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        match relay.ping().await {
                            Ok(_) => log::debug!("Ping {}", relay.url),
                            Err(err) => {
                                log::error!("Impossible to ping {}: {}", relay.url, err);
                                break;
                            }
                        }
                    }

                    if relay.status().await != RelayStatus::Terminated {
                        if let Err(err) = relay.disconnect().await {
                            log::error!("Impossible to disconnect {}: {}", relay.url, err);
                        }
                    }
                };

                #[cfg(feature = "blocking")]
                match new_current_thread() {
                    Ok(rt) => {
                        std::thread::spawn(move || {
                            rt.block_on(async move { func_relay_ping.await });
                            rt.shutdown_timeout(Duration::from_millis(100));
                        });
                    }
                    Err(e) => log::error!("Impossible to create new thread: {:?}", e),
                };

                #[cfg(not(feature = "blocking"))]
                tokio::task::spawn(func_relay_ping);
            }
            Err(err) => {
                self.set_status(RelayStatus::Disconnected).await;
                log::error!("Impossible to connect to {}: {}", url, err);
            }
        };
    }

    async fn send_relay_event(&self, relay_msg: RelayEvent) -> Result<()> {
        Ok(self.relay_sender.send(relay_msg).await?)
    }

    /// Ping relay
    async fn ping(&self) -> Result<()> {
        self.send_relay_event(RelayEvent::Ping).await
    }

    /// Disconnect from relay and set status to 'Disconnected'
    async fn disconnect(&self) -> Result<()> {
        self.send_relay_event(RelayEvent::Close).await
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub async fn terminate(&self) -> Result<()> {
        self.send_relay_event(RelayEvent::Terminate).await
    }

    pub async fn send_msg(&self, msg: ClientMessage) -> Result<()> {
        self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)))
            .await
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

    pub async fn connect_all(&mut self) -> Result<()> {
        for (relay_url, relay) in self.relays.clone().iter() {
            relay.connect().await;
            self.subscribe_relay(relay_url).await?;
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
