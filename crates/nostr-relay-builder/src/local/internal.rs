// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use async_utility::futures_util::stream::{self, SplitSink};
use async_utility::futures_util::{SinkExt, StreamExt};
use atomic_destructor::AtomicDestroyer;
use nostr::prelude::*;
use nostr_database::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use super::session::{RateLimiterResponse, Session, Tokens};
use super::util;
use crate::builder::{RateLimit, RelayBuilder};
use crate::error::Error;

type WsTx = SplitSink<WebSocketStream<TcpStream>, Message>;

#[derive(Debug, Clone)]
pub(super) struct InternalLocalRelay {
    addr: SocketAddr,
    database: Arc<DynNostrDatabase>,
    shutdown: broadcast::Sender<()>,
    /// Channel to notify new event received
    ///
    /// Every session will listen and check own subscriptions
    new_event: broadcast::Sender<Event>,
    rate_limit: RateLimit,
}

impl AtomicDestroyer for InternalLocalRelay {
    fn on_destroy(&self) {
        self.shutdown();
    }
}

impl InternalLocalRelay {
    pub async fn run(builder: RelayBuilder) -> Result<Self, Error> {
        // TODO: check if configured memory database with events option disabled

        // Get IP
        let ip: IpAddr = builder.addr.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        // Get port
        let port: u16 = match builder.port {
            Some(port) => port,
            None => util::find_available_port().await?,
        };

        // Compose local address
        let addr: SocketAddr = SocketAddr::new(ip, port);

        // Bind
        let listener: TcpListener = TcpListener::bind(addr).await?;

        // Channels
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        let (new_event, ..) = broadcast::channel(1024);

        // Compose relay
        let relay: Self = Self {
            addr,
            database: builder.database,
            shutdown: shutdown_tx,
            new_event,
            rate_limit: builder.rate_limit,
        };

        let r: Self = relay.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    output = listener.accept() => {
                        match output {
                            Ok((stream, addr)) => {
                                let r1: Self = r.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = r1.handle_connection(stream, addr).await {
                                        tracing::error!("{e}");
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Can't accept incoming connection: {e}");
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    },

                }
            }

            tracing::info!("Local relay listener loop terminated.");
        });

        Ok(relay)
    }

    #[inline]
    pub fn url(&self) -> String {
        format!("ws://{}", self.addr)
    }

    #[inline]
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }

    async fn handle_connection(&self, raw_stream: TcpStream, addr: SocketAddr) -> Result<()> {
        let mut shutdown_rx = self.shutdown.subscribe();
        let mut new_event = self.new_event.subscribe();

        let ws_stream = tokio_tungstenite::accept_async(raw_stream).await?;
        tracing::debug!("WebSocket connection established: {addr}");

        let (mut tx, mut rx) = ws_stream.split();

        let mut session: Session = Session {
            subscriptions: HashMap::new(),
            tokens: Tokens::new(self.rate_limit.notes_per_minute),
        };

        loop {
            tokio::select! {
                msg = rx.next() => {
                    match msg {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(json) => {
                                    tracing::trace!("Received {json}");
                                    self.handle_client_msg(&mut session, &mut tx, ClientMessage::from_json(json)?)
                                        .await?;
                                }
                                Message::Binary(..) => {
                                    let msg: RelayMessage =
                                        RelayMessage::notice("binary messages are not processed by this relay");
                                    if let Err(e) = self.send_msg(&mut tx, msg).await {
                                        tracing::error!("Can't send msg to client: {e}");
                                    }
                                }
                                Message::Ping(val) => {
                                    let _ = tx.send(Message::Pong(val)).await;
                                }
                                Message::Pong(..) => {}
                                Message::Close(..) => {}
                                Message::Frame(..) => {}
                            }
                        }
                        Some(Err(e)) => tracing::error!("Can't handle websocket msg: {e}"),
                        None => break,
                    }
                }
                event = new_event.recv() => {
                    if let Ok(event) = event {
                         // Iter subscriptions
                        for (id, filters) in session.subscriptions.iter() {
                            if filters.iter().any(|f| f.match_event(&event)) {
                                self.send_msg(&mut tx, RelayMessage::event(id.to_owned(), event.clone())).await?;
                            }
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        tracing::debug!("WebSocket connection terminated for {addr}");

        Ok(())
    }

    async fn handle_client_msg(
        &self,
        session: &mut Session,
        ws_tx: &mut WsTx,
        msg: ClientMessage,
    ) -> Result<()> {
        match msg {
            ClientMessage::Event(event) => {
                // Check rate limit
                if let RateLimiterResponse::Limited =
                    session.check_rate_limit(self.rate_limit.notes_per_minute)
                {
                    return self
                        .send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: format!(
                                    "{}: slow down",
                                    MachineReadablePrefix::RateLimited
                                ),
                            },
                        )
                        .await;
                }

                // Check if event already exists
                let event_status = self.database.check_id(&event.id).await?;
                if let DatabaseEventStatus::Saved | DatabaseEventStatus::Deleted = event_status {
                    return self
                        .send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: true,
                                message: format!(
                                    "{}: already have this event",
                                    MachineReadablePrefix::Duplicate
                                ),
                            },
                        )
                        .await;
                }

                if !event.verify_id() {
                    return self
                        .send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: format!(
                                    "{}: invalid event ID",
                                    MachineReadablePrefix::Invalid
                                ),
                            },
                        )
                        .await;
                }

                if !event.verify_signature() {
                    return self
                        .send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: format!(
                                    "{}: invalid event signature",
                                    MachineReadablePrefix::Invalid
                                ),
                            },
                        )
                        .await;
                }

                let msg: RelayMessage = match self.database.save_event(&event).await {
                    Ok(status) => {
                        if status {
                            let event_id = event.id;

                            // Broadcast to channel
                            self.new_event.send(*event)?;

                            // Reply to client
                            RelayMessage::Ok {
                                event_id,
                                status: true,
                                message: String::new(),
                            }
                        } else {
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: format!("{}: unknown", MachineReadablePrefix::Error),
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Can't save event into database: {e}");
                        RelayMessage::Ok {
                            event_id: event.id,
                            status: false,
                            message: format!("{}: database error", MachineReadablePrefix::Error),
                        }
                    }
                };

                self.send_msg(ws_tx, msg).await
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                // Check number of subscriptions
                if session.subscriptions.len() >= self.rate_limit.max_reqs
                    && !session.subscriptions.contains_key(&subscription_id)
                {
                    return self
                        .send_msg(
                            ws_tx,
                            RelayMessage::Closed {
                                subscription_id,
                                message: format!(
                                    "{}: too many REQs",
                                    MachineReadablePrefix::RateLimited
                                ),
                            },
                        )
                        .await;
                }

                // Update session subscriptions
                session
                    .subscriptions
                    .insert(subscription_id.clone(), filters.clone());

                // Query database
                let events = self.database.query(filters).await?;

                tracing::debug!(
                    "Found {} events for subscription '{subscription_id}'",
                    events.len()
                );

                let mut msgs: Vec<RelayMessage> = Vec::with_capacity(events.len() + 1);
                msgs.extend(
                    events
                        .into_iter()
                        .map(|e| RelayMessage::event(subscription_id.clone(), e)),
                );
                msgs.push(RelayMessage::eose(subscription_id));

                self.send_msgs(ws_tx, msgs).await?;

                Ok(())
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let count: usize = self.database.count(filters).await?;
                self.send_msg(ws_tx, RelayMessage::count(subscription_id, count))
                    .await
            }
            ClientMessage::Close(subscription_id) => {
                session.subscriptions.remove(&subscription_id);
                Ok(())
            }
            ClientMessage::Auth(_event) => {
                // TODO
                Ok(())
            }
            ClientMessage::NegOpen { .. }
            | ClientMessage::NegMsg { .. }
            | ClientMessage::NegClose { .. } => Ok(()),
        }
    }

    #[inline]
    async fn send_msg(&self, tx: &mut WsTx, msg: RelayMessage) -> Result<()> {
        tx.send(Message::Text(msg.as_json())).await?;
        Ok(())
    }

    #[inline]
    async fn send_msgs<I>(&self, tx: &mut WsTx, msgs: I) -> Result<()>
    where
        I: IntoIterator<Item = RelayMessage>,
    {
        let mut stream = stream::iter(msgs.into_iter()).map(|msg| Ok(Message::Text(msg.as_json())));
        tx.send_all(&mut stream).await?;
        Ok(())
    }
}
