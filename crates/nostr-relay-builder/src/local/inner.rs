// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::borrow::Cow;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use async_utility::futures_util::stream::{self, SplitSink};
use async_utility::futures_util::{SinkExt, StreamExt};
use async_wsocket::native::{self, Message, WebSocketStream};
use atomic_destructor::AtomicDestroyer;
use negentropy::{Id, Negentropy, NegentropyStorageVector};
use nostr_database::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Notify, Semaphore};

use super::session::{Nip42Session, RateLimiterResponse, Session, Tokens};
use super::util;
use crate::builder::{
    PolicyResult, QueryPolicy, RateLimit, RelayBuilder, RelayBuilderMode, RelayBuilderNip42,
    RelayTestOptions, WritePolicy,
};
use crate::error::Error;

type WsTx<S> = SplitSink<WebSocketStream<S>, Message>;

#[derive(Debug, Clone)]
pub(super) struct InnerLocalRelay {
    addr: SocketAddr,
    database: Arc<dyn NostrEventsDatabase>,
    shutdown: Arc<Notify>,
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
    write_policy: Vec<Arc<dyn WritePolicy>>,
    query_policy: Vec<Arc<dyn QueryPolicy>>,
    nip42: Option<RelayBuilderNip42>,
    test: RelayTestOptions,
}

impl AtomicDestroyer for InnerLocalRelay {
    fn on_destroy(&self) {
        self.shutdown();
    }
}

impl InnerLocalRelay {
    pub async fn new(builder: RelayBuilder) -> Result<Self, Error> {
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
        let (new_event, ..) = broadcast::channel(1024);

        let max_connections: usize = builder.max_connections.unwrap_or(Semaphore::MAX_PERMITS);

        // Compose relay
        Ok(Self {
            addr,
            database: builder.database,
            shutdown: Arc::new(Notify::new()),
            new_event,
            mode: builder.mode,
            rate_limit: builder.rate_limit,
            connections_limit: Arc::new(Semaphore::new(max_connections)),
            min_pow: builder.min_pow,
            #[cfg(feature = "tor")]
            hidden_service,
            write_policy: builder.write_plugins,
            query_policy: builder.query_plugins,
            nip42: builder.nip42,
            test: builder.test,
        })
    }

    pub async fn run(builder: RelayBuilder) -> Result<Self, Error> {
        let relay: Self = Self::new(builder).await?;
        relay.listen().await?;
        Ok(relay)
    }

    /// Start socket to listen for new websocket connections
    async fn listen(&self) -> Result<(), Error> {
        let listener: TcpListener = TcpListener::bind(&self.addr).await?;

        let r: Self = self.clone();
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
                    _ = r.shutdown.notified() => break,
                }
            }

            tracing::info!("Local relay listener loop terminated.");
        });

        Ok(())
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

    pub fn notify_event(&self, event: Event) -> bool {
        self.new_event.send(event).is_ok()
    }

    #[inline]
    pub fn shutdown(&self) {
        // There are at least 2 waiters
        self.shutdown.notify_waiters()
    }

    /// Handle already upgraded HTTP request
    pub(crate) async fn handle_upgraded_connection<S>(
        &self,
        stream: S,
        addr: SocketAddr,
    ) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        if let Some(unresponsive_connection) = self.test.unresponsive_connection {
            tokio::time::sleep(unresponsive_connection).await;
        }

        // Accept websocket
        let ws_stream = native::take_upgraded(stream).await;

        self.handle_websocket(ws_stream, addr).await?;

        Ok(())
    }

    /// Pass bare [TcpStream] for handling
    async fn handle_connection<S>(self, raw_stream: S, addr: SocketAddr) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        if let Some(unresponsive_connection) = self.test.unresponsive_connection {
            tokio::time::sleep(unresponsive_connection).await;
        }

        // Accept websocket
        let ws_stream = native::accept(raw_stream).await?;

        self.handle_websocket(ws_stream, addr).await?;

        Ok(())
    }

    /// Handle websocket connection
    async fn handle_websocket<S>(
        &self,
        ws_stream: WebSocketStream<S>,
        addr: SocketAddr,
    ) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        // Try to acquire connection limit
        let permit = self.connections_limit.try_acquire()?;

        tracing::debug!("WebSocket connection established: {addr}");

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
                                    self.handle_client_msg(&mut session, &mut tx, ClientMessage::from_json(json.as_bytes())?, &addr)
                                        .await?;
                                }
                                Message::Binary(..) => {
                                    let msg =
                                        RelayMessage::Notice(Cow::Borrowed("binary messages are not processed by this relay"));
                                    if let Err(e) = send_msg(&mut tx, msg).await {
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
                        for (subscription_id, filter) in session.subscriptions.iter() {
                            if filter.match_event(&event) {
                                send_msg(&mut tx, RelayMessage::Event{
                                    subscription_id: Cow::Borrowed(subscription_id),
                                    event: Cow::Borrowed(&event)
                                }).await?;
                            }
                        }
                    }
                }
                _ = self.shutdown.notified() => break,
            }
        }

        // Drop connection permit
        drop(permit);

        tracing::debug!("WebSocket connection terminated for {addr}");

        Ok(())
    }

    async fn handle_client_msg<S>(
        &self,
        session: &mut Session<'_>,
        ws_tx: &mut WsTx<S>,
        msg: ClientMessage<'_>,
        addr: &SocketAddr,
    ) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        match msg {
            ClientMessage::Event(event) => {
                // Check rate limit
                if let RateLimiterResponse::Limited =
                    session.check_rate_limit(self.rate_limit.notes_per_minute)
                {
                    return send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: Cow::Owned(format!(
                                    "{}: slow down",
                                    MachineReadablePrefix::RateLimited
                                )),
                            },
                        )
                        .await;
                }

                // Check POW
                if let Some(difficulty) = self.min_pow {
                    if !event.id.check_pow(difficulty) {
                        return send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: Cow::Owned(format!(
                                        "{}: required a difficulty >= {difficulty}",
                                        MachineReadablePrefix::Pow
                                    )),
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
                        send_msg(
                            ws_tx,
                            RelayMessage::Auth {
                                challenge: Cow::Owned(session.nip42.generate_challenge()),
                            },
                        )
                        .await?;

                        // Return error
                        return send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: Cow::Owned(format!(
                                        "{}: you must auth",
                                        MachineReadablePrefix::AuthRequired
                                    )),
                                },
                            )
                            .await;
                    }
                }

                // check write policy
                for policy in self.write_policy.iter() {
                    let event_id = event.id;
                    if let PolicyResult::Reject(m) = policy.admit_event(&event, addr).await {
                        return send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id,
                                    status: false,
                                    message: Cow::Owned(format!("{}: {}", MachineReadablePrefix::Blocked, m)),
                                },
                            )
                            .await;
                    }
                }

                // Check if event already exists
                let event_status = self.database.check_id(&event.id).await?;
                match event_status {
                    DatabaseEventStatus::Saved => {
                        return send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: true,
                                    message: Cow::Owned(format!(
                                        "{}: already have this event",
                                        MachineReadablePrefix::Duplicate
                                    )),
                                },
                            )
                            .await;
                    }
                    DatabaseEventStatus::Deleted => {
                        return send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: Cow::Owned(format!(
                                        "{}: this event is deleted",
                                        MachineReadablePrefix::Blocked
                                    )),
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
                        return send_msg(
                                ws_tx,
                                RelayMessage::Ok {
                                    event_id: event.id,
                                    status: false,
                                    message: Cow::Owned(format!(
                                        "{}: event not related to owner of this relay",
                                        MachineReadablePrefix::Blocked
                                    )),
                                },
                            )
                            .await;
                    }
                }

                if !event.verify_id() {
                    return send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: Cow::Owned(format!(
                                    "{}: invalid event ID",
                                    MachineReadablePrefix::Invalid
                                )),
                            },
                        )
                        .await;
                }

                if !event.verify_signature() {
                    return send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: Cow::Owned(format!(
                                    "{}: invalid event signature",
                                    MachineReadablePrefix::Invalid
                                )),
                            },
                        )
                        .await;
                }

                if event.kind.is_ephemeral() {
                    let event_id = event.id;

                    // Broadcast to channel
                    self.new_event.send(event.into_owned())?;

                    // Send OK message
                    return send_msg(
                            ws_tx,
                            RelayMessage::Ok {
                                event_id,
                                status: true,
                                message: Cow::Owned(String::new()),
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
                            self.new_event.send(event.into_owned())?;

                            // Reply to client
                            RelayMessage::Ok {
                                event_id,
                                status: true,
                                message: Cow::Owned(String::new()),
                            }
                        } else {
                            RelayMessage::Ok {
                                event_id: event.id,
                                status: false,
                                message: Cow::Owned(format!("{}: unknown", MachineReadablePrefix::Error)),
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Can't save event into database: {e}");
                        RelayMessage::Ok {
                            event_id: event.id,
                            status: false,
                            message: Cow::Owned(format!("{}: database error", MachineReadablePrefix::Error)),
                        }
                    }
                };

                send_msg(ws_tx, msg).await
            }
            ClientMessage::Req {
                subscription_id,
                filter,
            } => {
                // Check number of subscriptions
                if session.subscriptions.len() >= self.rate_limit.max_reqs
                    && !session.subscriptions.contains_key(&subscription_id)
                {
                    return send_msg(
                            ws_tx,
                            RelayMessage::Closed {
                                subscription_id,
                                message: Cow::Owned(format!(
                                    "{}: too many REQs",
                                    MachineReadablePrefix::RateLimited
                                )),
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
                        send_msg(
                            ws_tx,
                            RelayMessage::Auth {
                                challenge: Cow::Owned(session.nip42.generate_challenge()),
                            },
                        )
                        .await?;

                        // Return error
                        return send_msg(
                                ws_tx,
                                RelayMessage::Closed {
                                    subscription_id,
                                    message: Cow::Owned(format!(
                                        "{}: you must auth",
                                        MachineReadablePrefix::AuthRequired
                                    )),
                                },
                            )
                            .await;
                    }
                }

                // check query policy plugins
                for plugin in self.query_policy.iter() {
                    if let PolicyResult::Reject(msg) = plugin.admit_query(&filter, addr).await {
                        return send_msg(
                                ws_tx,
                                RelayMessage::Closed {
                                    subscription_id,
                                    message: Cow::Owned(format!("{}: {}", MachineReadablePrefix::Error, msg)),
                                },
                            )
                            .await;
                    }
                }

                let filter: Filter = filter.into_owned();

                // Check if subscription has IDs
                let ids_len: Option<usize> = filter.ids.as_ref().map(|ids| ids.len());

                // Query database
                let events: Events = self.database.query(filter.clone()).await?;
                let events_len: usize = events.len();

                tracing::debug!(
                    "Found {events_len} events for subscription '{subscription_id}'",
                );

                let mut json_msgs: Vec<String> = Vec::with_capacity(events_len + 1);
                json_msgs.extend(
                    events
                        .into_iter()
                        .map(|event| RelayMessage::Event {subscription_id: Cow::Borrowed(subscription_id.as_ref()), event: Cow::Owned(event)}.as_json()),
                );

                let eose_or_closed: RelayMessage = match ids_len {
                    // Requested IDs len is the same as the query output, close the subscription.
                    Some(ids_len) if ids_len == events_len => {
                        RelayMessage::Closed {
                            subscription_id,
                            message: Cow::Borrowed(""),
                        }
                    },
                    // The stored events are all served
                    _ => {
                        // Save the subscription
                        session
                            .subscriptions
                            .insert(subscription_id.clone().into_owned(), filter);

                        // Return EOSE
                        RelayMessage::EndOfStoredEvents(subscription_id)
                    }
                };
                json_msgs.push(eose_or_closed.as_json());

                // Send JSON messages
                send_json_msgs(ws_tx, json_msgs).await
            }
            ClientMessage::ReqMultiFilter { subscription_id, .. } => {
                send_msg(
                    ws_tx,
                    RelayMessage::Closed {
                        subscription_id,
                        message: Cow::Owned(format!("{}: multi-filter REQs aren't supported (https://github.com/nostr-protocol/nips/pull/1645)", MachineReadablePrefix::Unsupported)),
                    },
                ).await
            }
            ClientMessage::Count {
                subscription_id,
                filter,
            } => {
                let count: usize = self.database.count(filter.into_owned()).await?;
                send_msg(ws_tx, RelayMessage::Count { subscription_id, count }).await
            }
            ClientMessage::Close(subscription_id) => {
                session.subscriptions.remove(&subscription_id);
                Ok(())
            }
            ClientMessage::Auth(event) => match session.nip42.check_challenge(&event) {
                Ok(()) => {
                    send_msg(
                        ws_tx,
                        RelayMessage::Ok {
                            event_id: event.id,
                            status: true,
                            message: Cow::Owned(String::new()),
                        },
                    )
                    .await
                }
                Err(e) => {
                    send_msg(
                        ws_tx,
                        RelayMessage::Ok {
                            event_id: event.id,
                            status: false,
                            message: Cow::Owned(format!("{}: {e}", MachineReadablePrefix::AuthRequired)),
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
                let items = self.database.negentropy_items(filter.into_owned()).await?;

                tracing::debug!(
                    id = %subscription_id,
                    "Found {} items for negentropy reconciliation.",
                    items.len()
                );

                // Construct negentropy storage, add items and seal
                let mut storage = NegentropyStorageVector::with_capacity(items.len());
                for (id, timestamp) in items.into_iter() {
                    let id: Id = Id::from_byte_array(id.to_bytes());
                    storage.insert(timestamp.as_u64(), id)?;
                }
                storage.seal()?;

                // Construct negentropy client
                let mut negentropy = Negentropy::owned(storage, 60_000)?;

                // Reconcile
                let bytes: Vec<u8> = hex::decode(initial_message.as_ref())?;
                let message: Vec<u8> = negentropy.reconcile(&bytes)?;

                // Reply
                send_msg(
                    ws_tx,
                    RelayMessage::NegMsg {
                        subscription_id: Cow::Borrowed(&subscription_id),
                        message: Cow::Owned(hex::encode(message)),
                    },
                )
                .await?;

                // Update subscriptions
                session
                    .negentropy_subscription
                    .insert(subscription_id.into_owned(), negentropy);
                Ok(())
            }
            ClientMessage::NegMsg {
                subscription_id,
                message,
            } => {
                match session.negentropy_subscription.get_mut(&subscription_id) {
                    Some(negentropy) => {
                        // Reconcile
                        let bytes: Vec<u8> = hex::decode(message.as_ref())?;
                        let message = negentropy.reconcile(&bytes)?;

                        // Reply
                        send_msg(
                            ws_tx,
                            RelayMessage::NegMsg {
                                subscription_id,
                                message: Cow::Owned(hex::encode(message)),
                            },
                        )
                        .await
                    }
                    None => {
                        send_msg(
                            ws_tx,
                            RelayMessage::NegErr {
                                subscription_id,
                                message: Cow::Owned(format!(
                                    "{}: subscription not found",
                                    MachineReadablePrefix::Error
                                )),
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
}

#[inline]
async fn send_msg<S>(tx: &mut WsTx<S>, msg: RelayMessage<'_>) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    tx.send(Message::Text(msg.as_json().into())).await?;
    Ok(())
}

#[inline]
async fn send_json_msgs<I, S>(tx: &mut WsTx<S>, json_msgs: I) -> Result<()>
where
    I: IntoIterator<Item = String>,
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut stream = stream::iter(json_msgs.into_iter()).map(|msg| Ok(Message::Text(msg.into())));
    tx.send_all(&mut stream).await?;
    Ok(())
}
