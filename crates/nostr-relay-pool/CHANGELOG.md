# Changelog

<!-- All notable changes to this project will be documented in this file. -->

<!-- The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), -->
<!-- and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). -->

<!-- Template

## Unreleased

### Breaking changes

### Changed

### Added

### Fixed

### Removed

### Deprecated

-->

## Unreleased

### Breaking changes

- Rename `BoxSink` and `BoxStream` to `WebSocketSink` and `WebSocketStream` (https://github.com/rust-nostr/nostr/pull/1086)
- Replace `ReceiverStream` with `BoxStream` and make `stream` module private (https://github.com/rust-nostr/nostr/pull/1087)

### Changed

- Add checks to ensure REQ limits are respected before receiving the EOSE message (https://github.com/rust-nostr/nostr/pull/1024)

### Added

- Add `Relay::stream_events` method (https://github.com/rust-nostr/nostr/pull/1088)

### Fixed

- `ban_relay_on_mismatch` no longer requires `verify_subscriptions` to be enabled.

## v0.43.0 - 2025/07/28

### Breaking changes

- Drop NIP-11 support (https://github.com/rust-nostr/nostr/pull/950)
- Update `RelayPool::subscriptions` and `RelayPool::subscription` outputs (https://github.com/rust-nostr/nostr/pull/980)

### Changed

- Refine notification sending depending on event database saving status (https://github.com/rust-nostr/nostr/pull/911)
- Simplify received message logging (https://github.com/rust-nostr/nostr/pull/945)
- Trim incoming relay messages before processing
- Verify that the received events belong to a subscription (https://github.com/rust-nostr/nostr/pull/979)
- Disable the default event max size limit (https://github.com/rust-nostr/nostr/pull/996)
- Bump lru from 0.14 to 0.16

### Added

- Allow putting relays to sleep when idle (https://github.com/rust-nostr/nostr/pull/926)
- An option to ban relays that send events which don't match the subscription filter (https://github.com/rust-nostr/nostr/pull/981)
- Add `RelayOptions::verify_subscriptions` option (https://github.com/rust-nostr/nostr/pull/997)

### Fixed

- Fix panic after a broken pipe error in the relay connection (https://github.com/rust-nostr/nostr/pull/1007)

## v0.42.0 - 2025/05/20

### Breaking changes

- Drop support for deprecated negentropy protocol (https://github.com/rust-nostr/nostr/pull/853)

### Changed

- Bump `lru` from 0.13.0 to 0.14.0

### Added

- Add relay monitor (https://github.com/rust-nostr/nostr/pull/851)

### Fixed

- Fix wrong conversion from u8 to RelayStatus (https://github.com/rust-nostr/nostr/pull/878)

## v0.41.0 - 2025/04/15

### Breaking changes

- Remove `Error::Failed` variant
- Returns `Output` instead of an error if the message/event sending fails for all relays
- Add `reason` field to `AdmitStatus::Rejected` variant

### Changed

- Extend unit tests
- Better handling of `CLOSED` message for REQs (https://github.com/rust-nostr/nostr/pull/778)

### Added

- Add `Relay::ban` method
- Add `AdmitPolicy::admit_connection` method (https://github.com/rust-nostr/nostr/pull/831)

## v0.40.1 - 2025/03/24

### Fixed

- Fix `Relay::unsubscribe_all` method hangs

## v0.40.0 - 2025/03/18

### Breaking changes

- Drop `RelayFiltering`
- Remove `Relay` constructors
- Change `RelayPool::new` signature
- Allow setting the notification channel size of a single `Relay` using `RelayOptions`

### Changed

- Take event reference in `send_event` methods
- Use the relay ingester to perform actions
- Avoid spawning a task for every authentication request
- Use `std::sync::OnceLock` instead of `tokio::sync::OnceCell`
- Better handling of auto-closing subscription activity when fetching events (https://github.com/rust-nostr/nostr/pull/798)
- Reduce `WAIT_FOR_OK_TIMEOUT` to 10 secs
- Handle CLOSED message when syncing

### Added

- Event verification cache (https://github.com/rust-nostr/nostr/pull/746)
- Add `AdmitPolicy` trait (https://github.com/rust-nostr/nostr/pull/774)
- Add `ReqExitPolicy::WaitForEvents` variant (https://github.com/rust-nostr/nostr/pull/798)
- Add `RelayPoolBuilder`
- Add `RelayPool::is_shutdown` method

### Removed

- Remove min POW difficulty check in favor of `AdmitPolicy` trait

## v0.39.0 - 2025/01/31

### Breaking changes

- Change `Relay::connect` method signature
- Change `Relay::disconnect` method signature
- Change `RelayPool::disconnect` method signature
- Update `targets` arg type in `RelayPool::stream_events_targeted` method
- Update `RelayPool::remove_all_relays` method signature
- Update `RelayPool::force_remove_all_relays` method signature
- Update `RelayPool::shutdown` method signature

### Changed

- Bump `async-wsocket` to 0.13
- Bump `negentropy` to 0.5
- Update `Error::WebSocket` variant inner type
- Refactor negentropy sync methods
- Refactor relay pinger
- Refactor relay disconnect logic
- Check if pool was already shutdown when calling `RelayPool::shutdown`
- Avoid repeatedly locking the relay channel receiver
- Refactor `RelayPool::stream_events_targeted`
- Refactor relay removal logic and add unit tests
- Handle `close` WebSocket message
- Always close WebSocket connection when handlers terminate
- Better control over the handling of the termination request
- Ensure notification subscription in subscribe auto-close logic

### Added

- Add `Relay::try_connect`
- Add `Relay::wait_for_connection`
- Add `RelayPool::try_connect`
- Add `RelayPool::try_connect_relay`
- Add `RelayPool::wait_for_connection`
- Add WebSocket transport abstraction

## v0.38.0 - 2024/12/31

### Breaking changes

- Replace `Option<String>` with `String` in `Output::failed`

### Changed

- Bump `async-utility` to 0.3, `async-wsocket` to 0.12 and `atomic-destructor` to 0.3
- Add `PingTracker` and improve relay ping management
- Cleanup relay `Error` variants
- Acquire service watcher receiver outside the auto-connect loop
- Decrease `MAX_RETRY_INTERVAL` to 60 secs
- Rework retry interval calculation
- Improve shutdown docs
- Rename `FilterOptions` to `ReqExitPolicy`
- Log WebSocket connection error only if different from the last one
- Reduce atomic operations when cloning
- Derive `PartialOrd`, `Ord` and `Hash` for `RelayPoolNotification`

### Added

- Add `ReceiverStream`
- Add `SubscribeAutoCloseOptions::idle_timeout`

### Removed

- Remove `thiserror` and `tokio-stream` deps
- Remove minimum interval constraint in `RelayOptions::retry_interval`

### Deprecated

- Deprecated batch event methods
- Deprecate `FilterOptions`

## v0.37.0 - 2024/11/27

### Breaking changes

- Switch from async to sync message sending for `Relay`

### Changed

- Rework latency tracking
- Increase negentropy batch size down to 100
- Increase ping interval to 55 secs
- Increase max retry interval to 10 min
- Update retry interval calculation
- Try fetch relay information document only once every hour
- Not allow to add relays after `RelayPool` shutdown
- Rename `RelayOptions::retry_sec` to `RelayOptions::retry_interval`
- Rename `RelayOptions::adjust_retry_sec` to `RelayOptions::adjust_retry_interval`
- Request NIP11 document only after a successful WebSocket connection
- Immediately terminate relay connection on `Relay::disconnect` call
- Return error if relay doesn't exist when removing it

### Added

- Add relay reconnection and disconnection unit tests
- Add `RelayServiceFlags::GOSSIP` flag

### Fixed

- Fix relay can't manually connect if reconnection is disabled
- Fix reconnect loop not break if relay is disconnected while calling `Relay::disconnect`

### Removed

- Remove `Relay::support_negentropy`
- Remove `Error::NotConnectedStatusChanged` variant
- Remove `INBOX` and `OUTBOX` flags

### Deprecated

- Deprecate `RelayPoolNotification::RelayStatus` variant

## v0.36.0 - 2024/11/05

### Changed

- Changes in `RelayPool::remove_relay` behavior
- Allow multi-filter reconciliation
- Increase negentropy frame size limit to `60_000`
- Set default max relay message size to 5MB
- Return error when receive `RelayNotification::Shutdown` variant
- Rename `NegentropyOptions` and `NegentropyDirection` to `SyncOptions` and `SyncDirection`
- Join futures instead of spawning threads in `RelayPool` methods
- Reduce overhead by maintaining only one atomic reference count for `RelayConnectionStats` and `RelayFiltering` structs
- Switch to atomic operations for `RelayStatus`
- Replace `RwLock` with `OnceCell` for `external_notification_sender`
- Convert `InternalRelay::send_notification` and linked methods to sync
- Avoid `RelayNotification` cloning when not needed in `InternalRelay::send_notification`
- Avoid full `InnerRelay` clone when requesting NIP11 document
- Rework relay connection methods and auto-connection logic
- Increase `MAX_ADJ_RETRY_SEC` to 120 secs
- Return reference instead of cloned structs for some getter methods of `Relay` and `RelayPool`
- Removed unnecessary timeout during the shutdown notification process
- Deprecate `RelaySendOptions::skip_disconnected`
- Deprecate `RelayConnectionStats::uptime`
- Better error for health check if relay status is `Initialized`
- Connect in chunks if too many relays
- Dynamic channel size for streaming of events
- Allow to define a limit of relays allowed in `RelayPool`
- Refactor `Relay::batch_event` and `Relay::auth`
- Deprecate `RelaySendOptions`

### Added

- Add `RelayPool::force_remove_relay` method
- Add `RelayFiltering::overwrite_public_keys` method
- Add `RelayPool::sync_targeted`
- Add `Relay::reconcile_multi`
- Negentropy sync progress
- Add `RelayConnectionStats::success_rate`

### Removed

- Remove `RelayPool::reconcile_advanced`
- Remove `RelayPool::reconcile_with_items`

## v0.35.0 - 2024/09/19

### Changed

- Bump `async-wsocket` to `v0.8`
- Avoid unnecessary `Url` and `Relay` clone in `RelayPool` methods
- Avoid `Relay` clone in `RelayPool::connect_relay` method
- `RelayPool::send_event` and `RelayPool::batch_event` send only to relays with `WRITE` flag
- `RelayPool::subscribe_with_id`, `RelayPool::get_events_of` and `RelayPool::stream_events_of` REQ events only to relays with `READ` flag
- Bump `async-wsocket` to `v0.9`
- Improve `Relay::support_negentropy` method
- Change handle relay message log level from `error` to `warn`

### Added

- Add `RelayPool::disconnect_relay` method
- Add `RelayPool::relays_with_flag` and `RelayPool::all_relays`
- Add support to negentropy v1
- Add whitelist support

### Removed

- Remove high latency log
- Remove `Error::OneShotRecvError` variant

## v0.34.0 - 2024/08/15

### Changed

- Bump MSRV to v1.70.0
- Take mutex ownership instead of clone in `InternalRelayPool::get_events_from`
- Remove IDs collection from `InternalRelayPool::get_events_from`
- Better checks before perform queries or send messages to relays
- Bump `async-wsocket` to `v0.7`
- Get events only from remote relay when calling `get_events_of` or `get_events_from`

### Added

- Add `RelayPoolNotification::Authenticated` variant
- Add `RelayPool::save_subscription`

### Fixed

- Fix `Event` notification variant sent also for events sent by the SDK

## v0.33.0 - 2024/07/16

### Changed

- Use per-purpose dedicated relay channels
- Return relay urls to which `messages`/`events` have or not been sent for `send_*` and `batch_*` methods
- Return relay urls to which `subscription` have or not been success for `subscribe*` methods
- Rename `Relay::terminate` to `Relay::disconnect`
- Always send `RelayPoolNotification::Message` variant
- Return report for negentropy reconciliation

### Added


- Add `Output<T>` struct
- Add `Output<EventId>::id` and `Output<SubscriptionId>::id` methods
- Add dry run option for negentropy reconciliation

### Fixed

- Fix shutdown notification sent to external channel on `Relay::terminate` method call
- Fix `RelayPool::reconcile_advanced` method uses database items instead of the passed ones

### Removed

- Remove `RelayPoolNotification::Stop`
- Remove `RelayStatus::Stop`
- Remove `start` and `stop` methods

## v0.32.0 - 2024/06/07

### Changed

- Bump `atomic-destructor` to `v0.2`
- Increase default kind 3 event limit to `840000` bytes and `10000` tags
- Improve accuracy of latency calculation
- Refactoring and adj. `relay` internal module
- Log when websocket messages are successfully sent
- Always close the WebSocket when receiver loop is terminated
- Use timeout for WebSocket message sender
- Bump `async-wsocket` to `v0.5`

### Added

- Allow to set event limits per kind
- Log warn when high latency

### Fixed

- Fix relay doesn't auto reconnect in certain cases

## v0.31.0 - 2024/05/17

### Added

- Add `RelayPool::start`
- Add `NegentropyDirection` default

## v0.30.0 - 2024/04/15

### Changed

- Bump `async-wsocket` to `0.4`
- Return error if `urls` arg is empty in `InternalRelayPool::get_events_from`
- Allow to disable `RelayLimits`

### Added

- Add `Relay::handle_notifications`

## v0.29.4 - 2024/04/08

- Fix `InternalRelay::get_events_of_with_callback` timeout

## v0.29.3 - 2024/04/04

- Check filter limit in `InternalRelayPool::get_events_from`

## v0.29.2 - 2024/03/27

### Fixed

- Fix `get_events_of` issues

## v0.29.1 - 2024/03/26

### Fixed

- Fix spurious send_event timeout error (https://github.com/rust-nostr/nostr/pull/375)
