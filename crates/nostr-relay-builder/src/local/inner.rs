// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use async_utility::futures_util::stream::{self, SplitSink};
use async_utility::futures_util::{SinkExt, StreamExt};
use async_wsocket::native::{self, Message, WebSocketStream};
use atomic_destructor::AtomicDestroyer;
use negentropy::{Bytes, Id, NegentropyStorageVector};
use nostr_database::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Semaphore};

use super::session::{Nip42Session, RateLimiterResponse, Session, Tokens};
use super::util;
use crate::builder::{
    RateLimit, RelayBuilder, RelayBuilderMode, RelayBuilderNip42, RelayTestOptions,
};
use crate::error::Error;

type WsTx = SplitSink<WebSocketStream<TcpStream>, Message>;

#[derive(Debug, Clone)]
pub(super) struct InnerLocalRelay {
    addr: SocketAddr,
    database: Arc<dyn NostrEventsDatabase>,
    shutdown: broadcast::Sender<()>,
    /// Channel to notify new event received
    ///
    /// Every session will listen and check own subscriptions
    new_event: broadcast::Sender<Event>,
    mode: RelayBuilderMode,
    rate_limit: RateLimit,
    connections_limit: Arc<Semaphore>,
    min_pow: Option<u8>, // TODO: use AtomicU8 to allow to change it?
    #[cfg(feature = "tor")]
    hidden_service: Option<String>,
    nip42: Option<RelayBuilderNip42>,
    test: RelayTestOptions,
}

impl AtomicDestroyer for InnerLocalRelay {
    fn on_destroy(&self) {
        self.shutdown();
    }
}

impl InnerLocalRelay {
    pub async fn run(builder: RelayBuilder) -> Result<Self, Error> {
        // TODO: check if configured memory database with events option disabled

        // Get IP
        let ip: IpAddr = builder.addr.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        // Get port
        let port: u16 = match builder.port {
            Some(port) => port,
            None => util::find_available_port().await,
        };

        // Compose local address
        let addr: SocketAddr = SocketAddr::new(ip, port);

        // Bind
        let listener: TcpListener = TcpListener::bind(addr).await?;

        // If enabled, launch tor hidden service
        #[cfg(feature = "tor")]
        let hidden_service: Option<String> = match builder.tor {
            Some(opts) => {
                let service = native::tor::launch_onion_service(
                    opts.nickname,
                    addr,
                    80,
                    opts.custom_path.as_ref(),
                )
                .await?;
                service.onion_name().map(|n| format!("ws://{n}"))
            }
            None => None,
        };

        // Channels
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        let (new_event, ..) = broadcast::channel(1024);

        let max_connections: usize = builder.max_connections.unwrap_or(Semaphore::MAX_PERMITS);

        // Compose relay
        let relay: Self = Self {
            addr,
            database: builder.database,
            shutdown: shutdown_tx,
            new_event,
            mode: builder.mode,
            rate_limit: builder.rate_limit,
            connections_limit: Arc::new(Semaphore::new(max_connections)),
            min_pow: builder.min_pow,
            #[cfg(feature = "tor")]
            hidden_service,
            nip42: builder.nip42,
            test: builder.test,
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
                                        tracing::warn!("{e}");
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
    #[cfg(feature = "tor")]
    pub fn hidden_service(&self) -> Option<&str> {
        self.hidden_service.as_deref()
    }

    #[inline]
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }

    async fn handle_connection(&self, raw_stream: TcpStream, addr: SocketAddr) -> Result<()> {
        if let Some(unresponsive_connection) = self.test.unresponsive_connection {
            tokio::time::sleep(unresponsive_connection).await;
        }

        // Accept websocket
        let ws_stream = native::accept(raw_stream).await?;

        // Try to acquire connection limit
        let permit = self.connections_limit.try_acquire()?;

        tracing::debug!("WebSocket connection established: {addr}");

        let mut shutdown_rx = self.shutdown.subscribe();
        let mut new_event = self.new_event.subscribe();

        let (mut tx, mut rx) = ws_stream.split();

        let mut session: Session = Session {
            subscriptions: HashMap::new(),
            negentropy_subscription: HashMap::new(),
            nip42: Nip42Session::default(),
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

        // Drop connection permit
        drop(permit);

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

                // Check POW
                if let Some(difficulty) = self.min_pow {
                    if !event.id.check_pow(difficulty) {
                        return self
                            .send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: format!(
                                        "{}: required a difficulty >= {difficulty}",
                                        MachineReadablePrefix::Pow
                                    ),
                                },
                            )
                            .await;
                    }
                }

                // Check NIP42
                if let Some(nip42) = &self.nip42 {
                    // TODO: check if public key allowed

                    // Check mode and if it's authenticated
                    if nip42.mode.is_write() && !session.nip42.is_authenticated() {
                        // Generate and send AUTH challenge
                        self.send_msg(
                            ws_tx,
                            RelayMessage::Auth {
                                challenge: session.nip42.generate_challenge(),
                            },
                        )
                        .await?;

                        // Return error
                        return self
                            .send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: format!(
                                        "{}: you must auth",
                                        MachineReadablePrefix::AuthRequired
                                    ),
                                },
                            )
                            .await;
                    }
                }

                // Check if event already exists
                let event_status = self.database.check_id(&event.id).await?;
                match event_status {
                    DatabaseEventStatus::Saved => {
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
                    DatabaseEventStatus::Deleted => {
                        return self
                            .send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: format!(
                                        "{}: this event is deleted",
                                        MachineReadablePrefix::Blocked
                                    ),
                                },
                            )
                            .await;
                    }
                    DatabaseEventStatus::NotExistent => {}
                }

                // Check mode
                if let RelayBuilderMode::PublicKey(pk) = self.mode {
                    let authored: bool = event.pubkey == pk;
                    let tagged: bool = event.tags.public_keys().any(|p| p == &pk);

                    if !authored && !tagged {
                        return self
                            .send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: format!(
                                        "{}: event not related to owner of this relay",
                                        MachineReadablePrefix::Blocked
                                    ),
                                },
                            )
                            .await;
                    }
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

                if event.kind.is_ephemeral() {
                    let event_id = event.id;

                    // Broadcast to channel
                    self.new_event.send(*event)?;

                    // Send OK message
                    return self
                        .send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id,
                                status: true,
                                message: String::new(),
                            },
                        )
                        .await;
                }

                let msg: RelayMessage = match self.database.save_event(&event).await {
                    Ok(status) => {
                        // TODO: match status
                        if status.is_success() {
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

                // Check NIP42
                if let Some(nip42) = &self.nip42 {
                    // TODO: check if public key allowed

                    // Check mode and if it's authenticated
                    if nip42.mode.is_read() && !session.nip42.is_authenticated() {
                        // Generate and send AUTH challenge
                        self.send_msg(
                            ws_tx,
                            RelayMessage::Auth {
                                challenge: session.nip42.generate_challenge(),
                            },
                        )
                        .await?;

                        // Return error
                        return self
                            .send_msg(
                                ws_tx,
                                RelayMessage::Closed {
                                    subscription_id,
                                    message: format!(
                                        "{}: you must auth",
                                        MachineReadablePrefix::AuthRequired
                                    ),
                                },
                            )
                            .await;
                    }
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
            ClientMessage::Auth(event) => match session.nip42.check_challenge(&event) {
                Ok(()) => {
                    self.send_msg(
                        ws_tx,
                        RelayMessage::Ok {
                            event_id: event.id,
                            status: true,
                            message: String::new(),
                        },
                    )
                    .await
                }
                Err(e) => {
                    self.send_msg(
                        ws_tx,
                        RelayMessage::Ok {
                            event_id: event.id,
                            status: false,
                            message: format!("{}: {e}", MachineReadablePrefix::AuthRequired),
                        },
                    )
                    .await
                }
            },
            ClientMessage::NegOpen {
                subscription_id,
                filter,
                initial_message,
                ..
            } => {
                // TODO: check number of neg subscriptions

                // TODO: check nip42?

                // Query database
                let items = self.database.negentropy_items(*filter).await?;

                tracing::debug!(
                    id = %subscription_id,
                    "Found {} items for negentropy reconciliation.",
                    items.len()
                );

                // Construct negentropy storage, add items and seal
                let mut storage = NegentropyStorageVector::with_capacity(items.len());
                for (id, timestamp) in items.into_iter() {
                    let id: Id = Id::new(id.to_bytes());
                    storage.insert(timestamp.as_u64(), id)?;
                }
                storage.seal()?;

                // Construct negentropy client
                let mut negentropy = Negentropy::new(storage, 60_000)?;

                // Reconcile
                let bytes: Bytes = Bytes::from_hex(initial_message)?;
                let message: Bytes = negentropy.reconcile(&bytes)?;

                // Update subscriptions
                session
                    .negentropy_subscription
                    .insert(subscription_id.clone(), negentropy);

                // Reply
                self.send_msg(
                    ws_tx,
                    RelayMessage::NegMsg {
                        subscription_id,
                        message: message.to_hex(),
                    },
                )
                .await
            }
            ClientMessage::NegMsg {
                subscription_id,
                message,
            } => {
                match session.negentropy_subscription.get_mut(&subscription_id) {
                    Some(negentropy) => {
                        // Reconcile
                        let bytes: Bytes = Bytes::from_hex(message)?;
                        let message = negentropy.reconcile(&bytes)?;

                        // Reply
                        self.send_msg(
                            ws_tx,
                            RelayMessage::NegMsg {
                                subscription_id,
                                message: message.to_hex(),
                            },
                        )
                        .await
                    }
                    None => {
                        self.send_msg(
                            ws_tx,
                            RelayMessage::NegErr {
                                subscription_id,
                                message: format!(
                                    "{}: subscription not found",
                                    MachineReadablePrefix::Error
                                ),
                            },
                        )
                        .await
                    }
                }
            }
            ClientMessage::NegClose { subscription_id } => {
                session.negentropy_subscription.remove(&subscription_id);
                Ok(())
            }
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
