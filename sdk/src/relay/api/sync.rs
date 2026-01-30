use std::borrow::Cow;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Instant;

use async_utility::time;
use negentropy::{Id, Negentropy, NegentropyStorageVector};
use nostr::{ClientMessage, EventId, Filter, RelayMessage, SubscriptionId, Timestamp};
use tokio::sync::broadcast;

use crate::prelude::RelayNotification;
use crate::relay::constants::{
    NEGENTROPY_BATCH_SIZE_DOWN, NEGENTROPY_FRAME_SIZE_LIMIT, NEGENTROPY_HIGH_WATER_UP,
    NEGENTROPY_LOW_WATER_UP,
};
use crate::relay::{Error, Relay, SyncOptions};

/// Relay negentropy reconciliation summary
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncSummary {
    /// Events that were stored locally (missing on relay)
    pub local: HashSet<EventId>,
    /// Events that were stored on relay (missing locally)
    pub remote: HashSet<EventId>,
    /// Events that are **successfully** sent to relays during reconciliation
    pub sent: HashSet<EventId>,
    /// Event that are **successfully** received from relay during reconciliation
    pub received: HashSet<EventId>,
    /// Send failures
    pub send_failures: HashMap<EventId, String>,
    // /// Receive failures
    // pub receive: HashMap<EventId, Vec<String>>,
}

/// Sync events with relay
///
/// <https://github.com/nostr-protocol/nips/blob/master/77.md>
#[must_use = "Does nothing unless you await!"]
pub struct SyncEvents<'relay> {
    relay: &'relay Relay,
    filter: Filter,
    items: Option<Vec<(EventId, Timestamp)>>,
    opts: SyncOptions,
}

impl<'relay> SyncEvents<'relay> {
    #[inline]
    pub(crate) fn new(relay: &'relay Relay, filter: Filter) -> Self {
        Self {
            relay,
            filter,
            items: None,
            opts: SyncOptions::new(),
        }
    }

    /// Set sync items
    ///
    /// When items are provided, negentropy items are NOT fetched from the database.
    #[inline]
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (EventId, Timestamp)>,
    {
        self.items = Some(items.into_iter().collect());
        self
    }

    /// Set sync options
    #[inline]
    pub fn opts(mut self, opts: SyncOptions) -> Self {
        self.opts = opts;
        self
    }

    async fn exec(self) -> Result<SyncSummary, Error> {
        // Check if relay is operational
        self.relay.inner.ensure_operational()?;

        // Check if relay can read
        if !self.relay.inner.capabilities.can_read() {
            return Err(Error::ReadDisabled);
        }

        let items: Vec<(EventId, Timestamp)> = match self.items {
            Some(items) => items,
            None => {
                // Get negentropy items
                let database = self.relay.inner.state.database();
                database.negentropy_items(self.filter.clone()).await?
            }
        };

        let mut output: SyncSummary = SyncSummary::default();

        sync(
            self.relay,
            &self.filter,
            items.clone(),
            &self.opts,
            &mut output,
        )
        .await?;

        Ok(output)
    }
}

#[inline]
async fn send_neg_msg(relay: &Relay, id: &SubscriptionId, message: &str) -> Result<(), Error> {
    relay
        .send_msg(ClientMessage::NegMsg {
            subscription_id: Cow::Borrowed(id),
            message: Cow::Borrowed(message),
        })
        .await
}

#[inline]
async fn send_neg_close(relay: &Relay, id: &SubscriptionId) -> Result<(), Error> {
    relay
        .send_msg(ClientMessage::NegClose {
            subscription_id: Cow::Borrowed(id),
        })
        .await
}

#[inline]
fn neg_id_to_event_id(id: Id) -> EventId {
    EventId::from_byte_array(id.to_bytes())
}

#[inline(never)]
async fn handle_neg_msg<I>(
    relay: &Relay,
    subscription_id: &SubscriptionId,
    msg: Option<Vec<u8>>,
    curr_have_ids: I,
    curr_need_ids: I,
    opts: &SyncOptions,
    output: &mut SyncSummary,
    have_ids: &mut Vec<EventId>,
    need_ids: &mut Vec<EventId>,
    sync_done: &mut bool,
) -> nostr::Result<(), Error>
where
    I: Iterator<Item = EventId>,
{
    let mut counter: u64 = 0;

    // If event ID wasn't already seen, add to the HAVE IDs
    // Add to HAVE IDs only if `do_up` is true
    for id in curr_have_ids.into_iter() {
        if output.local.insert(id) && opts.do_up() {
            have_ids.push(id);
            counter += 1;
        }
    }

    // If event ID wasn't already seen, add to the NEED IDs
    // Add to NEED IDs only if `do_down` is true
    for id in curr_need_ids.into_iter() {
        if output.remote.insert(id) && opts.do_down() {
            need_ids.push(id);
            counter += 1;
        }
    }

    if let Some(progress) = &opts.progress {
        progress.send_modify(|state| {
            state.total += counter;
        });
    }

    match msg {
        Some(query) => send_neg_msg(relay, subscription_id, &hex::encode(query)).await,
        None => {
            // Mark sync as done
            *sync_done = true;

            // Send NEG-CLOSE message
            send_neg_close(relay, subscription_id).await
        }
    }
}

#[inline(never)]
async fn upload_neg_events(
    relay: &Relay,
    have_ids: &mut Vec<EventId>,
    in_flight_up: &mut HashSet<EventId>,
    opts: &SyncOptions,
) -> nostr::Result<(), Error> {
    // Check if it should skip the upload
    if !opts.do_up() || have_ids.is_empty() || in_flight_up.len() > NEGENTROPY_LOW_WATER_UP {
        return Ok(());
    }

    let mut num_sent = 0;

    while !have_ids.is_empty() && in_flight_up.len() < NEGENTROPY_HIGH_WATER_UP {
        if let Some(id) = have_ids.pop() {
            match relay.inner.state.database().event_by_id(&id).await {
                Ok(Some(event)) => {
                    in_flight_up.insert(id);
                    relay.send_msg(ClientMessage::event(event)).await?;
                    num_sent += 1;
                }
                Ok(None) => {
                    // Event not found
                }
                Err(e) => tracing::error!(
                    url = %relay.url(),
                    error = %e,
                    "Can't upload event."
                ),
            }
        }
    }

    // Update progress
    if let Some(progress) = &opts.progress {
        progress.send_modify(|state| {
            state.current += num_sent;
        });
    }

    if num_sent > 0 {
        tracing::info!(
            "Negentropy UP for '{}': {} events ({} remaining)",
            relay.url(),
            num_sent,
            have_ids.len()
        );
    }

    Ok(())
}

#[inline(never)]
async fn req_neg_events(
    relay: &Relay,
    need_ids: &mut Vec<EventId>,
    in_flight_down: &mut bool,
    down_sub_id: &SubscriptionId,
    opts: &SyncOptions,
) -> nostr::Result<(), Error> {
    // Check if it should skip the download
    if !opts.do_down() || need_ids.is_empty() || *in_flight_down {
        return Ok(());
    }

    let capacity: usize = cmp::min(need_ids.len(), NEGENTROPY_BATCH_SIZE_DOWN);
    let mut ids: Vec<EventId> = Vec::with_capacity(capacity);

    while !need_ids.is_empty() && ids.len() < NEGENTROPY_BATCH_SIZE_DOWN {
        if let Some(id) = need_ids.pop() {
            ids.push(id);
        }
    }

    tracing::info!(
        "Negentropy DOWN for '{}': {} events ({} remaining)",
        relay.url(),
        ids.len(),
        need_ids.len()
    );

    // Update progress
    if let Some(progress) = &opts.progress {
        progress.send_modify(|state| {
            state.current += ids.len() as u64;
        });
    }

    let filter = Filter::new().ids(ids);
    let msg: ClientMessage = ClientMessage::Req {
        subscription_id: Cow::Borrowed(down_sub_id),
        filters: vec![Cow::Borrowed(&filter)],
    };

    // Register an auto-closing subscription
    relay
        .inner
        .add_auto_closing_subscription(down_sub_id.clone(), vec![filter.clone()])
        .await;

    // Send msg
    if let Err(e) = relay.send_msg(msg).await {
        // Remove previously added subscription
        relay.inner.remove_subscription(down_sub_id).await;

        // Propagate error
        return Err(e);
    }

    *in_flight_down = true;

    Ok(())
}

/// Returns `true` if the events was in the `in_flight_up` collection.
#[inline(never)]
fn handle_neg_ok(
    relay: &Relay,
    in_flight_up: &mut HashSet<EventId>,
    event_id: EventId,
    status: bool,
    message: Cow<'_, str>,
    output: &mut SyncSummary,
) -> bool {
    if in_flight_up.remove(&event_id) {
        if status {
            output.sent.insert(event_id);
        } else {
            tracing::error!(
                url = %relay.url(),
                id = %event_id,
                msg = %message,
                "Can't upload event."
            );

            output.send_failures.insert(event_id, message.to_string());
        }

        true
    } else {
        false
    }
}

/// New negentropy protocol
#[inline(never)]
pub(super) async fn sync(
    relay: &Relay,
    filter: &Filter,
    items: Vec<(EventId, Timestamp)>,
    opts: &SyncOptions,
    output: &mut SyncSummary,
) -> nostr::Result<(), Error> {
    // Prepare the negentropy client
    let storage: NegentropyStorageVector = prepare_negentropy_storage(items)?;
    let mut negentropy: Negentropy<NegentropyStorageVector> =
        Negentropy::borrowed(&storage, NEGENTROPY_FRAME_SIZE_LIMIT)?;

    // Initiate reconciliation
    let initial_message: Vec<u8> = negentropy.initiate()?;

    // Subscribe
    let mut notifications = relay.inner.internal_notification_sender.subscribe();
    let mut temp_notifications = relay.inner.internal_notification_sender.subscribe();

    // Send the initial negentropy message
    let sub_id: SubscriptionId = SubscriptionId::generate();
    let open_msg: ClientMessage = ClientMessage::NegOpen {
        subscription_id: Cow::Borrowed(&sub_id),
        filter: Cow::Borrowed(filter),
        id_size: None,
        initial_message: Cow::Owned(hex::encode(initial_message)),
    };
    relay.send_msg(open_msg).await?;

    // Check if negentropy is supported
    check_negentropy_support(&sub_id, opts, &mut temp_notifications).await?;

    let mut in_flight_up: HashSet<EventId> = HashSet::new();
    let mut in_flight_down: bool = false;
    let mut sync_done: bool = false;
    let mut have_ids: Vec<EventId> = Vec::new();
    let mut need_ids: Vec<EventId> = Vec::new();
    let down_sub_id: SubscriptionId = SubscriptionId::generate();
    let mut last_relevant_msg: Instant = Instant::now();

    // Start reconciliation
    while let Ok(notification) = notifications.recv().await {
        if last_relevant_msg.elapsed() > opts.idle_timeout {
            return Err(Error::Timeout);
        }

        match notification {
            RelayNotification::Message { message } => {
                let is_relevant: bool = match message {
                    RelayMessage::NegMsg {
                        subscription_id,
                        message,
                    } => {
                        if subscription_id.as_ref() == &sub_id {
                            let mut curr_have_ids: Vec<Id> = Vec::new();
                            let mut curr_need_ids: Vec<Id> = Vec::new();

                            // Parse message
                            let query: Vec<u8> = hex::decode(message.as_ref())?;

                            // Reconcile
                            let msg: Option<Vec<u8>> = negentropy.reconcile_with_ids(
                                &query,
                                &mut curr_have_ids,
                                &mut curr_need_ids,
                            )?;

                            // Handle the message
                            handle_neg_msg(
                                relay,
                                &subscription_id,
                                msg,
                                curr_have_ids.into_iter().map(neg_id_to_event_id),
                                curr_need_ids.into_iter().map(neg_id_to_event_id),
                                opts,
                                output,
                                &mut have_ids,
                                &mut need_ids,
                                &mut sync_done,
                            )
                            .await?;

                            // Relevant to this sync
                            true
                        } else {
                            // Not relevant to this sync
                            false
                        }
                    }
                    RelayMessage::NegErr {
                        subscription_id,
                        message,
                    } => {
                        if subscription_id.as_ref() == &sub_id {
                            return Err(Error::RelayMessage(message.into_owned()));
                        } else {
                            // Not relevant to this sync
                            false
                        }
                    }
                    RelayMessage::Ok {
                        event_id,
                        status,
                        message,
                    } => handle_neg_ok(relay, &mut in_flight_up, event_id, status, message, output),
                    RelayMessage::Event {
                        subscription_id,
                        event,
                    } => {
                        if subscription_id.as_ref() == &down_sub_id {
                            output.received.insert(event.id);

                            // Relevant to this sync
                            true
                        } else {
                            // Not relevant to this sync
                            false
                        }
                    }
                    RelayMessage::EndOfStoredEvents(subscription_id) => {
                        if subscription_id.as_ref() == &down_sub_id {
                            in_flight_down = false;

                            // Remove subscription
                            relay.inner.remove_subscription(&down_sub_id).await;

                            // Close subscription
                            relay
                                .send_msg(ClientMessage::Close(Cow::Borrowed(&down_sub_id)))
                                .await?;

                            // Relevant to this sync
                            true
                        } else {
                            // Not relevant to this sync
                            false
                        }
                    }
                    RelayMessage::Closed {
                        subscription_id, ..
                    } => {
                        if subscription_id.as_ref() == &down_sub_id {
                            in_flight_down = false;

                            // NOTE: the subscription is removed in the `InnerRelay::handle_relay_message` method,
                            // so there is no need to try to remove it also here.

                            // Relevant to this sync
                            true
                        } else {
                            // Not relevant to this sync
                            false
                        }
                    }
                    _ => false,
                };

                // Send events
                upload_neg_events(relay, &mut have_ids, &mut in_flight_up, opts).await?;

                // Get events
                req_neg_events(
                    relay,
                    &mut need_ids,
                    &mut in_flight_down,
                    &down_sub_id,
                    opts,
                )
                .await?;

                // NOTE: update this after the uploading and requesting of the events, as it may require some time.
                if is_relevant {
                    last_relevant_msg = Instant::now();
                }
            }
            RelayNotification::RelayStatus { status } => {
                if status.is_disconnected() {
                    return Err(Error::NotConnected);
                }
            }
            _ => (),
        };

        if sync_done
            && have_ids.is_empty()
            && need_ids.is_empty()
            && in_flight_up.is_empty()
            && !in_flight_down
        {
            break;
        }
    }

    tracing::info!(url = %relay.url(), "Negentropy reconciliation terminated.");

    Ok(())
}

fn prepare_negentropy_storage(
    items: Vec<(EventId, Timestamp)>,
) -> nostr::Result<NegentropyStorageVector, Error> {
    // Compose negentropy storage
    let mut storage = NegentropyStorageVector::with_capacity(items.len());

    // Add items
    for (id, timestamp) in items.into_iter() {
        let id: Id = Id::from_byte_array(id.to_bytes());
        storage.insert(timestamp.as_secs(), id)?;
    }

    // Seal
    storage.seal()?;

    // Build negentropy client
    Ok(storage)
}

/// Check if negentropy is supported
#[inline(never)]
async fn check_negentropy_support(
    sub_id: &SubscriptionId,
    opts: &SyncOptions,
    temp_notifications: &mut broadcast::Receiver<RelayNotification>,
) -> nostr::Result<(), Error> {
    time::timeout(Some(opts.initial_timeout), async {
        while let Ok(notification) = temp_notifications.recv().await {
            if let RelayNotification::Message { message } = notification {
                match message {
                    RelayMessage::NegMsg {
                        subscription_id, ..
                    } => {
                        if subscription_id.as_ref() == sub_id {
                            break;
                        }
                    }
                    RelayMessage::NegErr {
                        subscription_id,
                        message,
                    } => {
                        if subscription_id.as_ref() == sub_id {
                            return Err(Error::RelayMessage(message.into_owned()));
                        }
                    }
                    RelayMessage::Notice(message) => {
                        if message == "ERROR: negentropy error: negentropy query missing elements" {
                            // The NEG-OPEN message is sent with 4 elements instead of 5
                            // If the relay return this error means that is not support new
                            // negentropy protocol
                            return Err(Error::Negentropy(
                                negentropy::Error::UnsupportedProtocolVersion,
                            ));
                        } else if message.contains("bad msg")
                            && (message.contains("unknown cmd")
                                || message.contains("negentropy")
                                || message.contains("NEG-"))
                        {
                            return Err(Error::NegentropyNotSupported);
                        } else if message.contains("bad msg: invalid message")
                            && message.contains("NEG-OPEN")
                        {
                            return Err(Error::UnknownNegentropyError);
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    })
    .await
    .ok_or(Error::Timeout)?
}

impl<'relay> IntoFuture for SyncEvents<'relay> {
    type Output = Result<SyncSummary, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'relay>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl_blocking!(SyncEvents<'_>);

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::time::Duration;

    use nostr_relay_builder::prelude::*;

    use super::*;
    use crate::relay::{SyncDirection, SyncOptions};

    #[tokio::test]
    async fn test_negentropy_sync() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        // Database
        let database = MemoryDatabase::with_opts(MemoryDatabaseOptions {
            events: true,
            max_events: None,
        });

        // Build events to store in the local database
        let local_events = vec![
            EventBuilder::text_note("Local 1")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::text_note("Local 2")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::new(Kind::Custom(123), "Local 123")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
        ];

        // Save an event to the local database
        for event in local_events.iter() {
            database.save_event(event).await.unwrap();
        }
        assert_eq!(database.count(Filter::new()).await.unwrap(), 3);

        // Relay
        let relay = Relay::builder(url).database(database.clone()).build();

        // Connect
        relay
            .try_connect()
            .timeout(Duration::from_secs(2))
            .await
            .unwrap();

        // Build events to send to the relay
        let relays_events = vec![
            // Event in common with the local database
            local_events[0].clone(),
            EventBuilder::text_note("Test 2")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::text_note("Test 3")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::new(Kind::Custom(123), "Test 4")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
        ];

        // Send events to the relays
        for event in relays_events.iter() {
            relay.send_event(event).await.unwrap();
        }

        // Sync
        let filter = Filter::new().kind(Kind::TextNote);
        let opts = SyncOptions::default().direction(SyncDirection::Both);
        let output = relay.sync(filter).opts(opts).await.unwrap();

        assert_eq!(
            output,
            SyncSummary {
                local: HashSet::from([local_events[1].id]),
                remote: HashSet::from([relays_events[1].id, relays_events[2].id]),
                sent: HashSet::from([local_events[1].id]),
                received: HashSet::from([relays_events[1].id, relays_events[2].id]),
                send_failures: HashMap::new(),
            }
        );
    }
}
