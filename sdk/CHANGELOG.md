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

- Replace `usize` with `u8` for gossip relay limits
- Remove `autoconnect` option from `ClientOptions`
- Change `Client::stream_events*` output to include the `RelayUrl` and `Result`, enabling callers to identify which relay sent the event or if a specific relay encountered an error (https://github.com/rust-nostr/nostr/pull/1156)
- Change `RelayPool::stream_events*` output to include the `RelayUrl` and `Result`, enabling callers to identify which relay sent the event or if a specific relay encountered an error (https://github.com/rust-nostr/nostr/pull/1156)
- Replace `RelayServiceFlags` with `RelayCapabilities`
- Remove `Client::reset` method
- Change `Client::try_connect` and `Relay::try_connect` signature (https://github.com/rust-nostr/nostr/pull/1229)
- Remove `Client::subscribe_with_id`, `Client::subscribe_to`, `Client::subscribe_with_id_to`, `Client::subscribe_targed` and `Relay::subscribe_with_id` (https://github.com/rust-nostr/nostr/pull/1232)
- Change `Client::subscribe` signature (https://github.com/rust-nostr/nostr/pull/1232) 
- Remove `Relay::sync_with_items` (https://github.com/rust-nostr/nostr/pull/1235)
- Replace `Reconciliation` struct with `SyncSummary` (https://github.com/rust-nostr/nostr/pull/1235)
- Change `Client::sync` and `Relay::sync` signature (https://github.com/rust-nostr/nostr/pull/1235)
- Make `RelayPool` private (https://github.com/rust-nostr/nostr/pull/1237)
- Replace `RelayPoolNotification` with `ClientNotification` (https://github.com/rust-nostr/nostr/pull/1237)
- Relays no longer inherit subscriptions of the pool (https://github.com/rust-nostr/nostr/pull/1237)
- Change `Client::relay` output to return `Result<Output<Relay>>`

### Changed

- Redesign `Client::add_relay` API (https://github.com/rust-nostr/nostr/pull/1227)
- Redesign `Client::remove_relay` API (https://github.com/rust-nostr/nostr/pull/1228)
- Redesign `Client::remove_relays` API (https://github.com/rust-nostr/nostr/pull/1229)
- Redesign `Client::connect`, `Client::try_connect` and `Relay::try_connect` APIs (https://github.com/rust-nostr/nostr/pull/1229)
  Redesign `Client::relays` API (https://github.com/rust-nostr/nostr/pull/1231)
- Redesign `Client::subscribe` and `Relay::subscribe` APIs (https://github.com/rust-nostr/nostr/pull/1232)
- Redesign `Client::stream_events` and `Relay::stream_events` APIs (https://github.com/rust-nostr/nostr/pull/1233)
- Redesign `Client::fetch_events` and `Relay::fetch_events` APIs (https://github.com/rust-nostr/nostr/pull/1234)
- Redesign `Client::sync` and `Relay::sync` APIs (https://github.com/rust-nostr/nostr/pull/1235)
- Redesign `Client::send_event` and `Relay::send_event` APIs (https://github.com/rust-nostr/nostr/pull/1236)
- Redesign `Client::unsubscribe` and `Relay::unsubscribe` APIs (https://github.com/rust-nostr/nostr/pull/1238)
- Redesign `Client::unsubscribe_all` and `Relay::unsubscribe_all` APIs (https://github.com/rust-nostr/nostr/pull/1239)

### Added

- Add `Client::is_shutdown`
- Add `RelayBuilder` struct and `Relay` constructors
- Re-add support for multi-filter REQ (https://github.com/rust-nostr/nostr/pull/1176)
- Add `Output::new` constructor and `Output::into_inner` method
- Add idle timeout for negentropy sync (https://github.com/rust-nostr/nostr/pull/1131)
- Add `GossipAllowedRelays` to `GossipOptions` to filter relays during selection (https://github.com/rust-nostr/nostr/pull/1128)

## v0.44.1 - 2025/11/09

### Fixed

- Fix doc building by removing `#![cfg_attr(docsrs, feature(doc_auto_cfg))]`

## v0.44.0 - 2025/11/06

### Breaking changes

- Remove `lmdb`, `ndb` and `indexeddb` features (https://github.com/rust-nostr/nostr/pull/1083)
- Replace `ReceiverStream` with `BoxedStream` (https://github.com/rust-nostr/nostr/pull/1087 and https://github.com/rust-nostr/nostr/pull/1121)

### Changed

- Fetch gossip NIP-17 list only if really needed (https://github.com/rust-nostr/nostr/pull/1090)
- Try to fetch only newer events when updating gossip lists (https://github.com/rust-nostr/nostr/pull/1090)
- Don't send kind 3 (contact list) to inbox relays when using gossip (https://github.com/rust-nostr/nostr/pull/1112)

### Added

- `Client::public_key` function to retrieve the public key (https://github.com/rust-nostr/nostr/pull/1028)

## v0.43.0 - 2025/07/28

### Breaking changes

- Update `Client::subscriptions` and `Client::subscription` outputs (https://github.com/rust-nostr/nostr/pull/980)

### Changed

- Extract at max 3 relays per NIP65 marker (https://github.com/rust-nostr/nostr/pull/951)

### Added

- Add `ClientOptions::sleep_when_idle` (https://github.com/rust-nostr/nostr/pull/959)
- add `verify_subscriptions` and `ban_relay_on_mismatch` to `ClientOptions` (https://github.com/rust-nostr/nostr/pull/998)

### Deprecated

- Deprecate `Options` in favor of `ClientOptions` (https://github.com/rust-nostr/nostr/pull/958)

## v0.42.0 - 2025/05/20

### Added

- Add `Options::pool`

### Deprecated

- Deprecate `Options::notification_channel_size`

## v0.41.0 - 2025/04/15

No notable changes in this release.

## v0.40.0 - 2025/03/18

### Breaking changes

- Change `Client::fetch_metadata` output
- Remove `Client::state`

### Changed

- Auto-update the gossip data when sending an event
- Avoid full clone of relays when only urls are needed

## v0.39.0 - 2025/01/31

### Breaking changes

- Update `Client::remove_all_relays` method signature
- Update `Client::force_remove_all_relays` method signature
- Update `Client::shutdown` method signature
- Change `Client::disconnect` method signature
- Change `Client::reset` method signature

### Added

- Add `Client::try_connect`
- Add `Client::try_connect_relay`
- Add `Client::wait_for_connection`
- Add `ClientBuilder::websocket_transport`

### Deprecated

- Deprecate `Options::req_filters_chunk_size`

## v0.38.0 - 2024/12/31

### Breaking changes

- Update `fetch_*` and `stream_*` methods signature

### Changed

- Refactor POW difficulty management

### Added

- Automatically resend event after NIP-42 authentication
- Add `Connection::embedded_tor_with_path`

### Fixed

- Fix NIP42 authentication for auto-closing REQ
- Fix min POW is not updated to already existing relays

### Removed

- Remove `thiserror` and `nwc` deps

### Deprecated

- Deprecate `timeout` option
- Deprecate `Options::difficulty` and `Client::update_difficulty`

## v0.37.0 - 2024/11/27

### Breaking changes

- Use `RelayUrl` struct instead of `Url` for relay urls
- Disable all default features
- Set `Client::from_builder` as private

### Changed

- Cleanup `Client` methods
- Fallback to READ relays if no relay list is set when breaking down filters

### Added

- Allow specifying relay pool notification channel size in `Options`
- Add support to NIP17 relay list

### Deprecated

- Deprecate `Client::with_opts`
- Deprecate `Options::connection_timeout`

## v0.36.0 - 2024/11/05

### Changed

- Deprecate `Client::get_events_of` and `Client::get_events_from` methods
- Use `Events` instead of `Vec<Event>` in fetch and query methods
- Rename `stream_events_of` to `stream_events`
- Deprecate `Client::reconcile` and `Client::reconcile_with`
- Use by default tor for onion relays if `tor` feature is enabled on non-mobile targets
- Return reference to `RelayPool` instead of clone in `Client:pool`
- Immediately return error if gossip filters are empty

### Added

- Add `Client::fetch_events` and `Client::fetch_events_from` methods
- Add `Client::sync` and `Client::sync_with` methods
- Add gossip support to `Client::sync`
- Add `Client::force_remove_all_relays`
- Add `Client::reset` and `switch-account` example

### Fixed

- Fix `filters empty` error when gossip option is enabled

## v0.35.0 - 2024/09/19

### Changed

- Bump `lnurl-pay` to `v0.6`
- Update `Client::gift_wrap` and `Client::gift_wrap_to` methods signature
- Document and rename `Client::metadata` to `Client::fetch_metadata`
- Update `Client::shutdown` method fingerprint
- Deprecate `Client::add_relay_with_opts` and `Client::add_relays`
- Deprecate `RelayPool::send_msg` and `RelayPool::batch_msg`
- Inherit pool subscriptions only when calling `Client::add_relay` or `Client::add_read_relay` methods

### Added

- Add `Client::add_discovery_relay`
- Add `Client::add_read_relay` and `Client::add_write_relay`
- Add `Client::stream_events_targeted`
- Add `Client::subscribe_targeted`
- Add gossip support to `Client::send_event`
- Add gossip support to `Client::get_events_of` and `Client::stream_events_of`
- Add gossip support to `Client::subscribe` and `Client::subscribe_with_id`

## v0.34.0 - 2024/08/15

### Changed

- Bump MSRV to v1.70.0
- Rename `Proxy` and `ProxyTarget` to `Connection` and `ConnectionTarget`
- Allow to skip slow relays
- Allow to specify the source of events for `Client::get_events_of` method
- Deprecate `Client::get_events_of_with_opts`

### Added

- Add `Client::gift_wrap_to` and `Client::send_private_msg_to`
- Add option to autoconnect relay on `Client::add_relay` method call
- Add support to embedded tor client
- Add `Options::max_avg_latency`
- Add `Client::stream_events_of` and `Client::stream_events_from` methods

### Removed

- Remove `Client::send_direct_msg`

## v0.33.0 - 2024/07/16

### Changed

- Allow to change auto authentication to relays option (NIP-42) after client initialization
- Retrieve contact list public keys only from the latest events
- Re-subscribe closed subscriptions after NIP-42 authentication

### Added

- Add `Client::unwrap_gift_wrap` method

### Fixed

- Fix NIP-42 client authentication

## v0.32.0 - 2024/06/07

### Changed

- Send NIP-42 event only to target relay

### Added

- Add support to automatic authentication to relays (NIP-42)

## v0.31.0 - 2024/05/17

### Changed

- Inline `Client`, `ClientBuilder` and `Options` methods
- Update `tokio` features
- Update visibility of `Options` field
- Remove zap split to support `rust-nostr` development from `Client::zap` method

### Added

- Add `Client::builder()`
- Add `Client::update_min_pow_difficulty` method
- Add `Client::connect_with_timeout`
- Add `Client::reconcile_with` and `Client::reconcile_advanced`
- Add `Client::subscribe_to` and `Client::subscribe_with_id_to` methods
- Add initial blacklist support
- Deprecate `Client::send_direct_msg`

## v0.30.0 - 2024/04/15

### Changed

- Bump `lnurl-pay` to `0.4`
- Improve `proxy` options
