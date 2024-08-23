// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! A mock relay for (unit) tests.

use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::{Duration, Instant};

use async_utility::futures_util::stream::{self, SplitSink};
use async_utility::futures_util::{SinkExt, StreamExt};
use atomic_destructor::{AtomicDestroyer, AtomicDestructor};
use nostr::prelude::*;
use nostr_database::{MemoryDatabase, MemoryDatabaseOptions, NostrDatabase, Order};
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

type WsTx = SplitSink<WebSocketStream<TcpStream>, Message>;

const MAX_REQS: usize = 20;
const NOTES_PER_MINUTE: i32 = 100;

/// Mock relay error
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error(transparent)]
    IO(#[from] io::Error),
    /// No port available
    #[error("No port available")]
    NoPortAvailable,
}

#[derive(Debug, Clone)]
struct RateLimit {
    pub notes_per_minute: i32,
    //pub whitelist: Option<Vec<String>>,
}

/// A mock relay for (unit) tests.
#[derive(Debug, Clone)]
pub struct MockRelay {
    inner: AtomicDestructor<InternalMockRelay>,
}

impl MockRelay {
    /// Run mock relay
    #[inline]
    pub async fn run() -> Result<Self, Error> {
        Ok(Self {
            inner: AtomicDestructor::new(InternalMockRelay::run().await?),
        })
    }

    /// Get url
    #[inline]
    pub fn url(&self) -> String {
        self.inner.url()
    }

    /// Shutdown relay
    #[inline]
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }
}

#[derive(Debug, Clone)]
struct InternalMockRelay {
    addr: SocketAddr,
    database: MemoryDatabase,
    shutdown: broadcast::Sender<()>,
    /// Channel to notify new event received
    ///
    /// Every session will listen and check own subscriptions
    new_event: broadcast::Sender<Event>,
    rate_limit: RateLimit,
}

impl AtomicDestroyer for InternalMockRelay {
    fn on_destroy(&self) {
        self.shutdown();
    }
}

impl InternalMockRelay {
    /// Run mock relay
    pub async fn run() -> Result<Self, Error> {
        // Find an available port
        let port: u16 = find_available_port().await?;

        let addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));

        // Bind
        let listener: TcpListener = TcpListener::bind(addr).await?;

        // Open database
        let opts = MemoryDatabaseOptions {
            events: true,
            max_events: None,
        };
        let database: MemoryDatabase = MemoryDatabase::with_opts(opts);

        // Channels
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        let (new_event, ..) = broadcast::channel(1024);

        // Compose relay
        let relay: Self = Self {
            addr,
            database,
            shutdown: shutdown_tx,
            new_event,
            rate_limit: RateLimit {
                notes_per_minute: NOTES_PER_MINUTE,
            },
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
                                tracing::error!("Can't accept incoming connection: {e}");
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    },

                }
            }

            tracing::info!("Mock relay listener loop terminated.");
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
                                    tracing::debug!("Received {json}");
                                    self.handle_client_msg(&mut session, &mut tx, ClientMessage::from_json(json)?)
                                        .await?;
                                }
                                Message::Binary(..) => {
                                    let msg: RelayMessage =
                                        RelayMessage::notice("binary messages are not processed by this relay");
                                    if let Err(e) = tx.send(Message::Text(msg.as_json())).await {
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
                if self
                    .database
                    .has_event_already_been_saved(&event.id)
                    .await?
                {
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
                if session.subscriptions.len() >= MAX_REQS
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
                let events = self.database.query(filters, Order::Desc).await?;

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

enum RateLimiterResponse {
    Allowed,
    Limited,
}

/// Tokens to keep track of session limits
struct Tokens {
    pub num: i32,
    pub last_note: Option<Instant>,
}

impl Tokens {
    #[inline]
    pub fn new(tokens: i32) -> Self {
        Self {
            num: tokens,
            last_note: None,
        }
    }
}

struct Session {
    pub subscriptions: HashMap<SubscriptionId, Vec<Filter>>,
    pub tokens: Tokens,
}

impl Session {
    /// `true` means that is rate limited
    pub fn check_rate_limit(&mut self, notes_per_minute: i32) -> RateLimiterResponse {
        match self.tokens.last_note {
            Some(last_note) => {
                let now: Instant = Instant::now();
                let mut diff: Duration = now - last_note;

                let min: Duration = Duration::from_secs(60);
                if diff > min {
                    diff = min;
                }

                let percent: f32 = (diff.as_secs() as f32) / 60.0;
                let new_tokens: i32 = (percent * notes_per_minute as f32).floor() as i32;
                self.tokens.num += new_tokens - 1;

                if self.tokens.num <= 0 {
                    self.tokens.num = 0;
                }

                if self.tokens.num >= notes_per_minute {
                    self.tokens.num = notes_per_minute - 1;
                }

                if self.tokens.num == 0 {
                    return RateLimiterResponse::Limited;
                }

                self.tokens.last_note = Some(now);

                RateLimiterResponse::Allowed
            }
            None => {
                self.tokens.last_note = Some(Instant::now());
                RateLimiterResponse::Allowed
            }
        }
    }
}

async fn find_available_port() -> Result<u16, Error> {
    for port in 8000..u16::MAX {
        if port_is_available(port).await {
            return Ok(port);
        }
    }

    Err(Error::NoPortAvailable)
}

#[inline]
async fn port_is_available(port: u16) -> bool {
    TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))
        .await
        .is_ok()
}
