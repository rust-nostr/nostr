# Changelog

<!-- All notable changes to this project will be documented in this file. -->

<!-- The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), -->
<!-- and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). -->

<!-- Template

## [Unreleased]

### Summary

### Breaking changes

### Changed

### Added

### Fixed

### Removed

### Deprecated

-->

## [Unreleased]

### Summary

### Breaking changes

* nostr: change `EventBuilder::gift_wrap` (and linked methods) args to take `extra_tags` instead of `expiration` ([erskingardner])
* nostr: change `EventBuilder::gift_wrap` (and linked methods) args to take an `EventBuilder` rumor instead of `UnsignedEvent` ([Yuki Kishimoto])
* nostr: change `EventBuilder::private_msg_rumor` arg to take `extra_tags` instead of `reply_to` ([Yuki Kishimoto])
* nostr: remove `tags` arg from `EventBuilder::new` ([Yuki Kishimoto])
* nostr: remove `tags` arg from `EventBuilder::text_note` ([Yuki Kishimoto])
* nostr: remove `tags` arg from `EventBuilder::long_form_text_note` ([Yuki Kishimoto])
* nostr: remove `tags` arg from `EventBuilder::job_request` ([Yuki Kishimoto])
* nostr: disable all default features except `std` ([Yuki Kishimoto])
* nostr: change `Timestamp::to_human_datetime` fingerprint ([Yuki Kishimoto])
* pool: switch from async to sync message sending for `Relay` ([Yuki Kishimoto])
* sdk: disable all default features ([Yuki Kishimoto])
* sdk: set `Client::from_builder` as private ([Yuki Kishimoto])
* ffi: convert `NostrSigner` trait to an object ([Yuki Kishimoto])

### Changed

* nostr: rewrite `e` tag de/serialization ([Yuki Kishimoto])
* pool: rework latency tracking ([Yuki Kishimoto])
* pool: increase negentropy batch size down to 100 ([Yuki Kishimoto])
* pool: increase ping interval to 55 secs ([Yuki Kishimoto])
* pool: increase max retry interval to 10 min ([Yuki Kishimoto])
* pool: update retry interval calculation ([Yuki Kishimoto])
* pool: try fetch relay information document only once every hour ([Yuki Kishimoto])
* pool: not allow to add relays after `RelayPool` shutdown ([Yuki Kishimoto])
* pool: rename `RelayOptions::retry_sec` to `RelayOptions::retry_interval` ([Yuki Kishimoto]) 
* pool: rename `RelayOptions::adjust_retry_sec` to `RelayOptions::adjust_retry_interval` ([Yuki Kishimoto])
* pool: request NIP11 document only after a successful WebSocket connection ([Yuki Kishimoto])
* pool: immediately terminate relay connection on `Relay::disconnect` call ([Yuki Kishimoto])
* pool: return error if relay not exists when removing it ([Yuki Kishimoto])
* sdk: cleanup `Client` methods
* relay-builder: port selection by using random port generation ([Yuki Kishimoto])
* lmdb: optimize vector initialization in unit tests ([Xiao Yu])
* nwc: increase default timeout to 60 secs ([Yuki Kishimoto])

### Added

* nostr: add NIP104 tag and event kinds ([erskingardner])
* nostr: add `SingleLetterTag::as_str` and `TagKind::as_str` ([Yuki Kishimoto])
* nostr: add `Kind::Comment` ([reyamir])
* nostr: add `EventBuilder::comment` ([reyamir])
* nostr: add uppercase field to `TagStandard::Coordinate` and `TagStandard::ExternalIdentity` variants ([reyamir])
* nostr: add `TagStandard::Quote` ([reyamir])
* nostr: add `Event::coordinate` ([Yuki Kishimoto])
* nostr: add `A/a` tags in `EventBuilder::comment` (NIP22) events ([Yuki Kishimoto])
* nostr: add NIP73 support ([Yuki Kishimoto])
* nostr: add `NostrSigner::backend` ([Yuki Kishimoto])
* nostr: add `EventBuilder::private_msg` ([Yuki Kishimoto])
* nostr: add `EventBuilder::tag` and `EventBuilder::tags` ([Yuki Kishimoto])
* pool: add relay reconnection and disconnection unit tests ([Yuki Kishimoto])
* sdk: allow to specify relay pool notification channel size in `Options` ([Yuki Kishimoto])
* relay-builder: add `RelayTestOptions` ([Yuki Kishimoto])
* connect: add `NostrConnect::non_secure_set_user_public_key` ([Yuki Kishimoto])
* ffi: add `make_private_msg` func ([Yuki Kishimoto])
* ffi: add `CustomNostrSigner` trait ([Yuki Kishimoto])
* book: add some examples ([RydalWater])

### Fixed

* nostr: fix `TagStandard` de/serialization unit tests ([Yuki Kishimoto])
* nostr: fix NIP90 kind ranges ([Janek])
* pool: fix relay can't manually connect if reconnection is disabled ([Yuki Kishimoto])
* pool: fix reconnect loop not break if relay is disconnected while calling `Relay::disconnect` ([Yuki Kishimoto])

### Removed

* nostr: remove `Marker::Custom` variant ([Yuki Kishimoto])
* pool: remove `Relay::support_negentropy` ([Yuki Kishimoto])
* pool: remove `Error::NotConnectedStatusChanged` variant ([Yuki Kishimoto])

### Deprecated

* nostr: deprecate `EventBuilder::add_tags` ([Yuki Kishimoto])
* pool: deprecate `RelayPoolNotification::RelayStatus` variant ([Yuki Kishimoto])
* sdk: deprecate `Client::with_opts` ([Yuki Kishimoto])
* sdk: deprecate `Options::connection_timeout` ([Yuki Kishimoto])

## [v0.36.0]

### Summary

Many, many improvements to `Relay` and `RelayPool` performance (reduced atomic operations and switched to async concurrency),
add `NostrSigner` trait, better methods and struct names (`fetch_events` instead of `get_events_of`, `sync` instead of `reconcile`,
`NostrConnect` instead of `Nip46Signer` and so on), add `LocalRelay` and allow to easily serve it as hidden onion service with the embedded
tor client, allow to keep track of negentropy sync progress, almost halved the weight of JavaScript SDK bindings (from ~6.3MB to ~3.6MB), some fixes and many more!

Note for Python, Kotlin, Swift and JavaScript devs: unfortunately I can't mark things as deprecated in bindings, so this release have many breaking changes, sorry :(
Note for devs who are using `nostr-protocol` (Python), `org.rust-nostr:nostr` (Kotlin), `nostr-swift` (Swift) or `@rust-nostr/nostr` (JavaScript) libraries: these packages are now deprecated. Only the `nostr-sdk` library will be released, which include everything that was in the `nostr` library.

### Changed

* Bump toolchain channel to `1.82.0`
* Convert `nostr-signer` crate to `nostr-connect` ([Yuki Kishimoto])
* nostr: move `TagsIndexes` into `Tags` struct ([Yuki Kishimoto])
* nostr: use `OnceCell` implementation from `std` lib instead of `once_cell` ([Yuki Kishimoto])
* nostr: remove redundant public key from repost events ([Yuki Kishimoto])
* nostr: change `impl Ord for Event` behaviour (descending order instead of ascending) ([Yuki Kishimoto])
* nostr: change `TagStandard::Relays` variant value from `Vec<UncheckedUrl>` to `Vec<Url>` ([Yuki Kishimoto])
* nostr: reserve capacity for tags when POW is enabled in `EventBuilder` ([Yuki Kishimoto])
* nostr: add `sign`, `sign_with_keys`, `sign_with_ctx`, `build` and `build_with_supplier` methods to `EventBuilder` ([Yuki Kishimoto])
* nostr: deprecate `to_event`, `to_event_with_ctx` and `to_unsigned_event` methods ([Yuki Kishimoto])
* relay-builder: refactor `Session::check_rate_limit` method ([Yuki Kishimoto])
* relay-builder: return error if event was deleted ([Yuki Kishimoto])
* pool: changes in `RelayPool::remove_relay` behavior ([Yuki Kishimoto])
* pool: allow multi-filter reconciliation ([Yuki Kishimoto])
* pool: increase negentropy frame size limit to `60_000` ([Yuki Kishimoto])
* pool: set default max relay message size to 5MB ([Yuki Kishimoto])
* pool: return error when receive `RelayNotification::Shutdown` variant ([Yuki Kishimoto])
* pool: rename `NegentropyOptions` and `NegentropyDirection` to `SyncOptions` and `SyncDirection` ([Yuki Kishimoto])
* pool: join futures instead of spawning threads in `RelayPool` methods ([Yuki Kishimoto])
* pool: reduce overhead by maintaining only one atomic reference count for `RelayConnectionStats` and `RelayFiltering` structs ([Yuki Kishimoto])
* pool: switch to atomic operations for `RelayStatus` ([Yuki Kishimoto])
* pool: replace `RwLock` with `OnceCell` for `external_notification_sender` ([Yuki Kishimoto])
* pool: convert `InternalRelay::send_notification` and linked methods to sync ([Yuki Kishimoto])
* pool: avoid `RelayNotification` cloning when not needed in `InternalRelay::send_notification` ([Yuki Kishimoto])
* pool: avoid full `InnerRelay` clone when requesting NIP11 document ([Yuki Kishimoto])
* pool: rework relay connection methods and auto-connection logic ([Yuki Kishimoto])
* pool: increase `MAX_ADJ_RETRY_SEC` to 120 secs ([Yuki Kishimoto])
* pool: return reference instead of cloned structs for some getter methods of `Relay` and `RelayPool` ([Yuki Kishimoto])
* pool: removed unnecessary timeout during the shutdown notification process ([Yuki Kishimoto])
* pool: deprecate `RelaySendOptions::skip_disconnected` ([Yuki Kishimoto])
* pool: deprecate `RelayConnectionStats::uptime` ([Yuki Kishimoto])
* pool: better error for health check if relay status is `Initialized` ([Yuki Kishimoto])
* pool: connect in chunks if too many relays ([Yuki Kishimoto])
* pool: dynamic channel size for streaming of events ([Yuki Kishimoto])
* pool: allow to define a limit of relays allowed in `RelayPool` ([Yuki Kishimoto])
* pool: refactor `Relay::batch_event` and `Relay::auth` ([Yuki Kishimoto])
* pool: deprecate `RelaySendOptions` ([Yuki Kishimoto])
* sdk: deprecate `Client::get_events_of` and `Client::get_events_from` methods ([Yuki Kishimoto])
* sdk: use `Events` instead of `Vec<Event>` in fetch and query methods ([Yuki Kishimoto])
* sdk: rename `stream_events_of` to `stream_events` ([Yuki Kishimoto])
* sdk: deprecate `Client::reconcile` and `Client::reconcile_with` ([Yuki Kishimoto])
* sdk: use by default tor for onion relays if `tor` feature is enabled on non-mobile targets ([Yuki Kishimoto])
* sdk: return reference to `RelayPool` instead of clone in `Client:pool` ([Yuki Kishimoto])
* sdk: immediately return error if gossip filters are empty ([Yuki Kishimoto])
* signer: auto enable `nip44` feature if `nip59` is enabled ([Yuki Kishimoto])
* connect: rename `Nip46Signer` to `NostrConnect` ([Yuki Kishimoto])
* database: improve `BTreeCappedSet` ([Yuki Kishimoto])
* database: not save invalid event deletion ([Yuki Kishimoto])
* lmdb: not save event deletion ([Yuki Kishimoto])
* lmdb: return iterator instead of vector in `Lmdb::single_filter_query` ([Yuki Kishimoto])
* lmdb: mark event as deleted only if database have the target event ([Yuki Kishimoto])
* signer: bootstrap NIP46 signer on demand ([Yuki Kishimoto])
* bindings(nostr): adj. `tag` module ([Yuki Kishimoto])
* ffi: merge `nostr-ffi` in `nostr-sdk-ffi` ([Yuki Kishimoto])
* js: merge `nostr-js` into `nostr-sdk-js` ([Yuki Kishimoto])
* js: change `opt-level` to `z` ([Yuki Kishimoto])

### Added

* nostr: add `TagKind::Client` variant ([Yuki Kishimoto])
* nostr: add some shorthand constructors for `TagKind::SingleLetter` ([Yuki Kishimoto])
* nostr: add `Tags` struct ([Yuki Kishimoto])
* nostr: add `d` tag extraction test from `Tags` ([Yuki Kishimoto])
* nostr: add `TagStandard::GitClone` and `TagKind::Clone` variants ([Yuki Kishimoto])
* nostr: add `TagStandard::GitCommit` and `TagKind::Commit` variants ([Yuki Kishimoto])
* nostr: add `TagStandard::GitEarliestUniqueCommitId` variant ([Yuki Kishimoto])
* nostr: add `TagStandard::GitMaintainers` and `TagKind::Maintainers` variants ([Yuki Kishimoto])
* nostr: add `TagStandard::Web` and `TagKind::Web` variants ([Yuki Kishimoto])
* nostr: add `EventBuilder::git_repository_announcement` ([Yuki Kishimoto])
* nostr: add `EventBuilder::git_issue` ([Yuki Kishimoto])
* nostr: add `EventBuilder::git_patch` ([Yuki Kishimoto])
* nostr: add `Tag::reference` constructor ([Yuki Kishimoto])
* nostr: add `nip59::make_seal` function ([Yuki Kishimoto])
* nostr: add `NostrSigner` trait ([Yuki Kishimoto])
* database: add `Backend::is_persistent` method ([Yuki Kishimoto])
* database: add `Events` struct ([Yuki Kishimoto])
* relay-builder: add `LocalRelay` and `RelayBuilder` ([Yuki Kishimoto])
* relay-builder: allow to serve local relay as hidden service ([Yuki Kishimoto])
* relay-builder: allow to set number of max connections allowed ([Yuki Kishimoto])
* relay-builder: add `RelayBuilderMode` ([Yuki Kishimoto])
* relay-builder: add min POW difficulty option to `RelayBuilder` ([Yuki Kishimoto])
* relay-builder: handle ephemeral events ([Yuki Kishimoto])
* pool: add `RelayPool::force_remove_relay` method ([Yuki Kishimoto])
* pool: add `RelayFiltering::overwrite_public_keys` method ([Yuki Kishimoto])
* pool: add `RelayPool::sync_targeted` ([Yuki Kishimoto])
* pool: add `Relay::reconcile_multi` ([Yuki Kishimoto])
* pool: negentropy sync progress ([Yuki Kishimoto])
* pool: add `RelayConnectionStats::success_rate` ([Yuki Kishimoto])
* sdk: add `Client::fetch_events` and `Client::fetch_events_from` methods ([Yuki Kishimoto])
* sdk: add `Client::sync` and `Client::sync_with` methods ([Yuki Kishimoto])
* sdk: add gossip support to `Client::sync` ([Yuki Kishimoto])
* sdk: add `Client::force_remove_all_relays` ([Yuki Kishimoto])
* sdk: add `Client::reset` and `switch-account` example ([Yuki Kishimoto])
* signer: add `NostrSigner::gift_wrap` ([Yuki Kishimoto])
* zapper: add `WebLNZapper` struct (moved from `nostr-webln` crate) ([Yuki Kishimoto])
* ffi(nostr): add `tag_kind_to_string` func ([Yuki Kishimoto])
* ffi(nostr): add `Tag::kind_str` method ([Yuki Kishimoto])
* ffi(nostr): impl `Display` for `Kind` ([Yuki Kishimoto])
* js(nostr): add `JsKind::_to_string` method ([Yuki Kishimoto])
* js(nostr): expose `from_nostr_uri` and `to_nostr_uri` for `PublicKey` and `EventId` ([Yuki Kishimoto])
* cli: show negentropy sync progress ([Yuki Kishimoto])
* book: add some examples ([RydalWater])
* book: add NIP17 example ([rodant])

### Fixed

* nostr: adj. `NostrConnectURI` de/serialization according to NIP46 ([Yuki Kishimoto])
* connect: fix `NostrConnect` according to NIP46
* lmdb: add missing commit method call in `Store::delete` ([Yuki Kishimoto])
* lmdb: fix unit tests ([Yuki Kishimoto])
* lmdb: fix `Store::save_event` issues ([Yuki Kishimoto])
* sdk: fix `filters empty` error when gossip option is enabled ([Yuki Kishimoto])

### Removed

* Remove deprecated ([Yuki Kishimoto])
* pool: remove `RelayPool::reconcile_advanced` ([Yuki Kishimoto])
* pool: remove `RelayPool::reconcile_with_items` ([Yuki Kishimoto])
* webln: remove `nostr-webln` crate ([Yuki Kishimoto])
* sqlite: remove `nostr-sqlite` crate ([Yuki Kishimoto])

## [v0.35.0]

### Summary

Add gossip model support, deprecate `SQLite` database in favor of `LMDB` 
(fork of [pocket](https://github.com/mikedilger/pocket) database),
add support to negentropy v1 (old version is still supported!), add `MockRelay` (a local disposable relay for tests),
allow usage of embedded tor client on mobile devices, many improvements, bugs fix and more!

### Changed

* nostr: bump `bitcoin` to `v0.32` ([Yuki Kishimoto])
* nostr: bump `base64` to `v0.22` ([Yuki Kishimoto])
* nostr: deprecate `Event::from_value` ([Yuki Kishimoto])
* nostr: deprecate `Tag::as_vec` ([Yuki Kishimoto])
* nostr: re-write `RawRelayMessage` parsing ([Yuki Kishimoto])
* nostr: update `Event` fields ([Yuki Kishimoto])
* nostr: deprecate `Event::is_*` kind related methods ([Yuki Kishimoto])
* nostr: change `TryIntoUrl::Err` to `Infallible` for `Url` ([Yuki Kishimoto])
* nostr: change `Event::verify_id` and `Event::verify_signature` fingerprint ([Yuki Kishimoto])
* nostr: impl custom `Debug`, `PartialEq` and `Eq` for `Keys` ([Yuki Kishimoto])
* nostr: impl `PartialOrd`, `Ord` and `Hash` for `Keys` ([Yuki Kishimoto])
* nostr: change `Keys::secret_key` and `Keys::sign_schnorr` methods fingerprint ([Yuki Kishimoto])
* nostr: deprecate `Keys::generate_without_keypair` ([Yuki Kishimoto])
* nostr: change NIP26 functions fingerprint ([Yuki Kishimoto])
* nostr: improve `NostrWalletConnectURI` parsing ([Yuki Kishimoto])
* nostr: update `EventBuilder::job_feedback` method fingerprint ([Yuki Kishimoto])
* nostr: deprecate `EventBuilder::to_pow_event` ([Yuki Kishimoto])
* nostr: impl `Display` for `MachineReadablePrefix` ([Yuki Kishimoto])
* nostr: improve `Keys` docs ([Yuki Kishimoto])
* nostr: change visibility of `public_key` field in `Keys` struct ([Yuki Kishimoto])
* nostr: deprecate `Keys::public_key_ref` ([Yuki Kishimoto])
* nostr: use `OsRng` instead of `ThreadRng` for `SECP256K1` global context and schnorr signing ([Yuki Kishimoto])
* nostr: improve `Timestamp::to_human_datetime` performance ([Yuki Kishimoto])
* nostr: deprecate `EventId::owned` ([Yuki Kishimoto])
* nostr: convert `EventId::all_zeroes` to const function ([Yuki Kishimoto])
* nostr: convert `Timestamp::from_secs` to const function ([Yuki Kishimoto])
* nostr: deprecate `Kind::as_u32` and `Kind::as_u64` ([Yuki Kishimoto])
* database: update `NostrDatabase` supertraits ([Yuki Kishimoto])
* database: impl `Clone` for `MemoryDatabase` ([Yuki Kishimoto])
* database: update `NostrDatabase::event_by_id` fingerprint ([Yuki Kishimoto])
* relay-builder: bump `tokio-tungstenite` to `v0.24` ([Yuki Kishimoto])
* pool: bump `async-wsocket` to `v0.8` ([Yuki Kishimoto])
* pool: avoid unnecessary `Url` and `Relay` clone in `RelayPool` methods ([Yuki Kishimoto])
* pool: avoid `Relay` clone in `RelayPool::connect_relay` method ([Yuki Kishimoto])
* pool: `RelayPool::send_event` and `RelayPool::batch_event` send only to relays with `WRITE` flag ([Yuki Kishimoto])
* pool: `RelayPool::subscribe_with_id`, `RelayPool::get_events_of` and `RelayPool::stream_events_of` REQ events only to relays with `READ` flag ([Yuki Kishimoto])
* pool: bump `async-wsocket` to `v0.9` ([Yuki Kishimoto])
* pool: improve `Relay::support_negentropy` method ([Yuki Kishimoto])
* pool: change handle relay message log level from `error` to `warn` ([Yuki Kishimoto])
* signer: update NIP04 and NIP44 methods signature ([Yuki Kishimoto])
* webln: bump `webln` to `v0.3` ([Yuki Kishimoto])
* sqlite: deprecate `SQLiteDatabase` in favor of LMDB ([Yuki Kishimoto])
* sdk: bump `lnurl-pay` to `v0.6` ([Yuki Kishimoto])
* sdk: update `Client::gift_wrap` and `Client::gift_wrap_to` methods signature ([Yuki Kishimoto])
* sdk: document and rename `Client::metadata` to `Client::fetch_metadata` ([Janek])
* sdk: update `Client::shutdown` method fingerprint ([Yuki Kishimoto])
* sdk: deprecate `Client::add_relay_with_opts` and `Client::add_relays` ([Yuki Kishimoto])
* sdk: deprecate `RelayPool::send_msg` and `RelayPool::batch_msg` ([Yuki Kishimoto])
* sdk: inherit pool subscriptions only when calling `Client::add_relay` or `Client::add_read_relay` methods ([Yuki Kishimoto])
* ffi(nostr): impl `Display` for `Coordinate` ([Yuki Kishimoto])
* ffi(sdk): change `Connection::embedded_tor` fingerprint for `android` and `ios` targets ([Yuki Kishimoto])
* cli: rename `open` command to `shell` ([Yuki Kishimoto])
* cli: rename `serve-signer` command to `bunker` ([Yuki Kishimoto])

### Added

* nostr: impl `TryFrom<Vec<Tag>>` for `LiveEvent` ([w3irdrobot])
* nostr: add `Tag::as_slice` ([Yuki Kishimoto])
* nostr: add `NostrWalletConnectURI::parse` ([Yuki Kishimoto])
* nostr: add `JobFeedbackData` struct ([Yuki Kishimoto])
* nostr: add `EventBuilder::pow` method ([Yuki Kishimoto])
* nostr: add `TagKind::custom` constructor ([Yuki Kishimoto])
* nostr: add `Timestamp::from_secs` ([Yuki Kishimoto])
* nostr: add `EventId::from_byte_array` ([Yuki Kishimoto])
* nostr: add `Timestamp::min` and `Timestamp::max` ([Yuki Kishimoto])
* nostr: add `nip65::extract_owned_relay_list` ([Yuki Kishimoto])
* nostr: add `Kind::from_u16` ([Yuki Kishimoto])
* database: add `DatabaseHelper::fast_query` ([Yuki Kishimoto])
* database: add `NostrDatabase::check_id` ([Yuki Kishimoto])
* database: add `NostrDatabaseExt::relay_lists` ([Yuki Kishimoto])
* lmdb: add LMDB storage backend ([Yuki Kishimoto])
* relay-builder: add `MockRelay` ([Yuki Kishimoto])
* pool: add `RelayPool::disconnect_relay` method ([Yuki Kishimoto])
* pool: add `RelayPool::relays_with_flag` and `RelayPool::all_relays` ([Yuki Kishimoto])
* pool: add support to negentropy v1 ([Yuki Kishimoto])
* pool: add whitelist support ([Yuki Kishimoto])
* sdk: add `Client::add_discovery_relay` ([Yuki Kishimoto])
* sdk: add `Client::add_read_relay` and `Client::add_write_relay` ([Yuki Kishimoto])
* sdk: add `Client::stream_events_targeted` ([Yuki Kishimoto])
* sdk: add `Client::subscribe_targeted` ([Yuki Kishimoto])
* sdk: add gossip support to `Client::send_event` ([Yuki Kishimoto])
* sdk: add gossip support to `Client::get_events_of` and `Client::stream_events_of` ([Yuki Kishimoto])
* sdk: add gossip support to `Client::subscribe` and `Client::subscribe_with_id` ([Yuki Kishimoto])
* bindings(nostr): expose `as_pretty_json` for some structs ([Yuki Kishimoto])
* bindings(sdk): expose `Client::fetch_metadata` ([Yuki Kishimoto])
* bindings(sdk): expose `Client::pool` method ([Yuki Kishimoto])
* ffi(nostr): expose `Kind::is_*` methods ([Yuki Kishimoto])
* ffi(sdk): expose `MockRelay` ([Yuki Kishimoto])
* js(nostr): add `Kind` object ([Yuki Kishimoto])
* js(nostr): expose `getNip05Profile` function ([Yuki Kishimoto])
* js(nostr): expose missing methods to `JsCoordinate` ([Yuki Kishimoto])
* js(sdk): expose `RelayPool::relays` ([Yuki Kishimoto])
* cli: add `serve` command ([Yuki Kishimoto])
* cli: add shell history ([Yuki Kishimoto])
* book: add some examples ([RydalWater])

### Fixed

* nostr: fix `TagStanderd::to_vec` ([nanikamado])
* nostr: fix broken intra doc links ([Yuki Kishimoto])
* nostr: fix `JsonUtil::try_as_pretty_json` method ([Yuki Kishimoto])
* nostr: fix `Kind::is_regular` method ([Yuki Kishimoto])

### Removed

* Drop support for `rocksdb` ([Yuki Kishimoto])
* nostr: remove `bech32` from the public API ([Yuki Kishimoto])
* nostr: remove `Keys::from_public_key` ([Yuki Kishimoto])
* nostr: remove `tracing` dep ([Yuki Kishimoto])
* nostr: remove impl `fmt::Display` for `SecretKey` ([Yuki Kishimoto])
* database: remove `has_event_already_been_saved`, `has_event_already_been_seen` and `has_event_id_been_deleted` methods from `NostrDatabase` ([Yuki Kishimoto])
* database: remove `Err` from `NostrDatabase` ([Yuki Kishimoto])
* database: remove `NostrDatabase::bulk_import` ([Yuki Kishimoto])
* database: remove `DatabaseError::NotFound` variant ([Yuki Kishimoto])
* database: remove `DatabaseError::Nostr` variant ([Yuki Kishimoto])
* database: remove `Order` enum ([Yuki Kishimoto])
* database: remove `order` arg from `NostrDatabase::query` ([Yuki Kishimoto])
* pool: remove high latency log ([Yuki Kishimoto])
* pool: remove `Error::OneShotRecvError` variant ([Yuki Kishimoto])
* zapper: remove `Err` from `NostrZapper` and unnecessary variants from `ZapperError` ([Yuki Kishimoto])
* js(nostr): remove `Keys::vanity` ([Yuki Kishimoto])
* cli: remove `reverse` flag from `query` command ([Yuki Kishimoto])

## [v0.34.0]

### Summary

Add embedded tor client support, allow to open databases with a limited capacity (automatically discard old events when max capacity is reached),
add `Client::stream_events_of` as alternative method to `Client::get_events_of` (stream events instead of waiting for `EOSE` and collect into a list),
add search capability (NIP50) support to `Filter::match_event` and databases, add NIP31 and NIP70 support,
add option to autoconnect relay on `Client::add_relay` method call (currently disabled by default), rework the `get_events_of` methods behaviour for 
better consistency (`RelayPool::get_events_of` and `Relay::get_events_of` get events only from remote relay/s while
`Client::get_events_of` allow to choose the source of events: `database`, `relays` or `both`), bugs fix and more!

### Changed

* Bump MSRV to v1.70.0 ([Yuki Kishimoto])
* Bump toolchain channel to `1.80.1` ([Yuki Kishimoto])
* nostr: deprecate `Event::author_ref` and `Event::iter_tags` ([Yuki Kishimoto])
* nostr: calculate `EventId` in `EventBuilder::to_unsigned_event_with_supplier` ([Yuki Kishimoto])
* nostr: ensure that NIP59 rumor has `EventId` ([Yuki Kishimoto])
* nostr: update `PartialEvent` methods ([Yuki Kishimoto])
* nostr: change `EventBuilder::award_badge` fingerprint ([Yuki Kishimoto])
* nostr: add NIP50 support to `Filter::match_event` method ([Yuki Kishimoto])
* nostr: remove `Arc<T>` from `OnceCell<T>` in `Event` and `Tag` ([Yuki Kishimoto])
* nostr: move `sig` field from `PartialEvent` to `MissingPartialEvent` ([Yuki Kishimoto])
* nostr: better `Debug` trait impl for `EventId`, `PublicKey` and `Tag` ([Yuki Kishimoto])
* nostr: improve `SubscriptionId::generate_with_rng` ([Yuki Kishimoto])
* pool: take mutex ownership instead of clone in `InternalRelayPool::get_events_from` ([Yuki Kishimoto])
* pool: remove IDs collection from `InternalRelayPool::get_events_from` ([Yuki Kishimoto])
* pool: better checks before perform queries or send messages to relays ([Yuki Kishimoto])
* pool: bump `async-wsocket` to `v0.7` ([Yuki Kishimoto])
* pool: get events only from remote relay when calling `get_events_of` or `get_events_from` ([Yuki Kishimoto])
* database: avoid to copy `EventId` in `Event::decode` ([Yuki Kishimoto])
* database: use `Vec` instead of `BTreeSet` as inner value for `TagIndexValues` ([Yuki Kishimoto])
* database: rework `DatabaseIndexes` and rename to `DatabaseHelper` ([Yuki Kishimoto])
* database: allow to set max capacity to `DatabaseHelper` ([Yuki Kishimoto])
* database: speedup helper bulk load ([Yuki Kishimoto])
* database: set a default logic for `NostrDatabase::negentropy_items` ([Yuki Kishimoto])
* sdk: rename `Proxy` and `ProxyTarget` to `Connection` and `ConnectionTarget` ([Yuki Kishimoto])
* sdk: allow to skip slow relays ([Yuki Kishimoto])
* sdk: allow to specify the source of events for `Client::get_events_of` method ([Yuki Kishimoto])
* sdk: deprecate `Client::get_events_of_with_opts` ([Yuki Kishimoto])
* sqlite: use `ValueRef` instead of owned one ([Yuki Kishimoto])
* cli: improve `sync` command ([Yuki Kishimoto])
* cli: allow to specify relays in `open` command ([Yuki Kishimoto])

### Added

* nostr: add NIP31 support ([Yuki Kishimoto])
* nostr: add NIP70 support ([Yuki Kishimoto])
* nostr: add `EventId::LEN` const ([Yuki Kishimoto])
* nostr: add `UnsignedEvent::ensure_id` method ([Yuki Kishimoto])
* nostr: add missing `payload` arg to `EventBuilder::job_result` ([Yuki Kishimoto])
* nostr: add `ConversationKey::new` ([Yuki Kishimoto])
* nostr: add `Request::multi_pay_invoice` constructor ([Yuki Kishimoto])
* nostr: add `Jsonutil::as_pretty_json` and `JsonUtil::try_as_pretty_json` methods ([Yuki Kishimoto])
* nostr: add `Coordinate::has_identifier` ([Yuki Kishimoto])
* pool: add `RelayPoolNotification::Authenticated` variant ([Yuki Kishimoto])
* pool: add `RelayPool::save_subscription` ([Yuki Kishimoto])
* sqlite/rocksdb/indexeddb: allow to open database with limited capacity ([Yuki Kishimoto])
* sdk: add `Client::gift_wrap_to` and `Client::send_private_msg_to` ([reyamir])
* sdk: add option to autoconnect relay on `Client::add_relay` method call ([Yuki Kishimoto])
* sdk: add support to embedded tor client ([Yuki Kishimoto])
* sdk: add `Options::max_avg_latency` ([Yuki Kishimoto])
* sdk: add `Client::stream_events_of` and `Client::stream_events_from` methods ([Yuki Kishimoto])
* ffi(nostr): add `EventBuilder::seal` constructor ([Yuki Kishimoto])
* cli: add `generate` command ([Yuki Kishimoto])
* cli: add `json` flag to `query` command ([Yuki Kishimoto])
* book: add some python examples ([RydalWater])

### Fixed

* pool: fix `Event` notification variant sent also for events sent by the SDK ([Yuki Kishimoto])
* database: fix indexes `QueryPattern` ([Yuki Kishimoto])
* database: fix query issue due to wrong tag value order ([Yuki Kishimoto])

### Removed

* Remove deprecated methods/functions ([Yuki Kishimoto])
* nostr: remove support for `nrelay` NIP19 entity ([Yuki Kishimoto])
* nostr: remove support for NIP44 v1 ([Yuki Kishimoto])
* nostr: remove `EventBuilder::encrypted_direct_msg` ([Yuki Kishimoto])
* database: remove `TempEvent` ([Yuki Kishimoto])
* database: remove `NostrDatabase::event_ids_by_filters` ([Yuki Kishimoto])
* sdk: remove `Client::send_direct_msg` ([Yuki Kishimoto])
* cli: remove `tracing-subscriber` dep ([Yuki Kishimoto])

## [v0.33.0]

### Summary

Better outputs for send/batch/reconcile methods (ex. you can now easily know where a message/event is successfully published and where/why failed),
allow to change NIP42 option after client initialization, increase max stack size for JS bindings to prevent "memory access out of bounds" error,
expose more objects/methods for JS bindings, dry run option for negentropy reconciliation, get NIP46 relay from NIP05 profile, 
bug fixes (NIP-42 auth not works correctly, NIP-46 "ACK" message not handled, ...) and more!

### Changed

* Bump `uniffi` to `v0.28.0` ([Yuki Kishimoto])
* nostr: rename NIP-51 `EventBuilder` set constructors and `Kind` variants ([Yuki Kishimoto])
* nostr: small adj. to NIP-47 `ListTransactionsRequestParams` and `LookupInvoiceResponseResult` structs ([Yuki Kishimoto])
* nostr: add `identifier` arg to NIP-51 `EventBuilder` set constructors ([Yuki Kishimoto])
* nostr: change `nip65::extract_relay_list` fingerprint ([Yuki Kishimoto])
* nostr: avoid allocation where possible in NIP-05 module ([Yuki Kishimoto])
* nostr: get NIP-46 relays from NIP-05 address ([DanConwayDev])
* nostr: deprecate `EventBuilder::encrypted_direct_msg` ([Yuki Kishimoto])
* pool: use per-purpose dedicated relay channels ([Yuki Kishimoto])
* pool: return relay urls to which `messages`/`events` have or not been sent for `send_*` and `batch_*` methods ([Yuki Kishimoto])
* pool: return relay urls to which `subscription` have or not been success for `subscribe*` methods ([Yuki Kishimoto])
* pool: rename `Relay::terminate` to `Relay::disconnect` ([Yuki Kishimoto])
* pool: always send `RelayPoolNotification::Message` variant ([Yuki Kishimoto])
* pool: return report for negentropy reconciliation ([Yuki Kishimoto])
* signer: use `limit(0)` instead of `since` for `Nip46Signer` subscription filter ([Yuki Kishimoto])
* signer: deprecate `NostrConnectRemoteSigner::nostr_connect_uri` and `Nip46Signer::nostr_connect_uri` ([Yuki Kishimoto])
* sdk: allow to change auto authentication to relays option (NIP-42) after client initialization ([Yuki Kishimoto])
* sdk: retrieve contact list public keys only from the latest events ([Xiao Yu])
* sdk: re-subscribe closed subscriptions after NIP-42 authentication ([Yuki Kishimoto])
* bindings(nostr): allow to specify coordinates in `EventBuilder::delete` constructor ([Yuki Kishimoto])
* ffi(sdk): convert `RelayPool::handle_notifications` method to async/future ([Yuki Kishimoto])
* js: increase max stack size to `0x1E84800` bytes (32 MiB) ([Yuki Kishimoto])
* js(nostr): adj. method names to camelcase format ([Yuki Kishimoto])

### Added

* nostr: add `EventBuilder::interest_set` ([Yuki Kishimoto])
* nostr: add `title`, `image` and `description` constructors to `Tag` ([Yuki Kishimoto])
* nostr: add `Timestamp::zero` and `Timestamp::is_zero` methods ([Yuki Kishimoto])
* nostr: add `Nip05Profile` struct ([Yuki Kishimoto])
* nostr: add `nip05::profile` function ([Yuki Kishimoto])
* nostr: add `LEN` const to `PublicKey`, `SecretKey` and `EncryptedSecretKey` ([Yuki Kishimoto])
* nostr: add `Report::Malware` variant ([Daniel Cadenas])
* nostr: add `coordinate` methods to `Filter` struct ([DanConwayDev])
* nostr: add NIP-34 kinds ([DanConwayDev])
* nostr: add `MachineReadablePrefix` enum ([Yuki Kishimoto])
* nostr: add `ClientMessage::is_auth` ([Yuki Kishimoto])
* pool: add `Output<T>` struct ([Yuki Kishimoto])
* pool: add `Output<EventId>::id` and `Output<SubscriptionId>::id` methods ([Yuki Kishimoto])
* pool: add dry run option for negentropy reconciliation ([Yuki Kishimoto])
* signer: add `NostrSigner::unwrap_gift_wrap` method ([Yuki Kishimoto])
* signer: add `bunker_uri` method to NIP-46 client and signer ([Yuki Kishimoto])
* sdk: add `Client::unwrap_gift_wrap` method ([Yuki Kishimoto])
* js(nostr): complete `JsFilter` struct ([Yuki Kishimoto])
* js(sdk): partially expose `JsRelayPool` ([Yuki Kishimoto])
* book: add some python examples ([RydalWater])

### Fixed

* nostr: fix NIP-47 `list_transactions` response deserialization ([Yuki Kishimoto] and [lnbc1QWFyb24])
* pool: fix shutdown notification sent to external channel on `Relay::terminate` method call ([Yuki Kishimoto])
* pool: fix `RelayPool::reconcile_advanced` method uses database items instead of the passed ones ([Yuki Kishimoto])
* signer: add missing NIP-46 connect "ACK" message handling ([Yuki Kishimoto])
* sdk: fix NIP-42 client authentication ([Yuki Kishimoto])
* js: fix "RuntimeError: memory access out of bounds" WASM error ([Yuki Kishimoto])

### Removed

* pool: remove `RelayPoolNotification::Stop` ([Yuki Kishimoto])
* pool: remove `RelayStatus::Stop` ([Yuki Kishimoto])
* Remove all `start` and `stop` methods ([Yuki Kishimoto])

## [v0.32.0]

### Summary

Added `async`/`future` support to Python, Kotlin and Swift, added automatic authentication to relays (NIP-42, can be deactivated in client options),
improvements to relay limits, many bug fixes (relays not auto reconnect, wrong query order for SQLite,
tokio panic when using SQLite database in bindings) and more!

Note for kotlin devs: from this release the packages will be published at `org.rust-nostr` instead of `io.github.rust-nostr`.

### Changed

* Bump `atomic-destructor` to `v0.2` ([Yuki Kishimoto])
* Bump `uniffi` to `v0.27.2` ([Yuki Kishimoto])
* nostr: ignore malformed public keys during NIP19 event (`nevent`) parsing ([Yuki Kishimoto])
* nostr: update `Event::pubic_keys` and `Event_event_ids` methods ([Yuki Kishimoto])
* nostr: adj. NIP-10 support ([Yuki Kishimoto])
* nostr: change fingerprint of `nip05::verify` ([Yuki Kishimoto])
* nostr: rework `TagStandard::parse` ([Yuki Kishimoto])
* nostr: add `a` tag to zap receipts ([benthecarman])
* nostr: change NIP-07 `Error::Wasm` variant value from `JsValue` to `String` ([Yuki Kishimoto])
* nostr: update `EventBuilder::live_event_msg` fingerprint ([Yuki Kishimoto])
* nostr: set `kind` arg in `EventBuilder::reaction_extended` as optional ([Yuki Kishimoto])
* pool: increase default kind 3 event limit to `840000` bytes and `10000` tags ([Yuki Kishimoto])
* pool: improve accuracy of latency calculation ([Yuki Kishimoto])
* pool: refactoring and adj. `relay` internal module ([Yuki Kishimoto])
* pool: log when websocket messages are successfully sent ([Yuki Kishimoto])
* pool: always close the WebSocket when receiver loop is terminated ([Yuki Kishimoto])
* pool: use timeout for WebSocket message sender ([Yuki Kishimoto])
* pool: bump `async-wsocket` to `v0.5` ([Yuki Kishimoto])
* sdk: send NIP-42 event only to target relay ([Yuki Kishimoto])
* sqlite: bump `rusqlite` to `v0.31` ([Yuki Kishimoto])
* nwc: change `NWC::new` and `NWC::with_opts` fingerprint ([Yuki Kishimoto])
* ffi: migrate kotlin packages to `org.rust-nostr` ([Yuki Kishimoto])
* bindings(sdk): log git hash after logger initialization ([Yuki Kishimoto])
* ffi(nostr): set default args values where possible ([Yuki Kishimoto])
* ffi(nostr): convert `verify_nip05` and `get_nip05_profile` to async functions ([Yuki Kishimoto])
* ffi(nostr): convert `RelayInformationDocument::get` to async ([Yuki Kishimoto])
* ffi(nostr): merge `Keys::from_mnemonic_*` constructors into `Keys::from_menmonic` ([Yuki Kishimoto])
* ffi(sdk): add `async/future` support (convert from blocking to async) ([Yuki Kishimoto])
* ffi(sdk): no longer spawn a thread when calling `handle_notifications` ([Yuki Kishimoto])
* js(sdk): change `JsNostrZapper::nwc` fingerprint ([Yuki Kishimoto])
* js(sdk): rename `JsNip46Signer::new` to `JsNip46Signer::init` ([Yuki Kishimoto])
* ci: build python wheels for `manylinux_2_28_x86_64` ([Yuki Kishimoto])

### Added

* nostr: add `Tag::is_root` method ([Xiao Yu])
* nostr: add `JsonUtil::try_as_json` method ([Yuki Kishimoto])
* nostr: add `public_key` field to `TagStandard::Event` ([Yuki Kishimoto])
* nostr: add support to `nrelay` NIP-19 entity ([Yuki Kishimoto])
* nostr: add `Event::get_tag_content` method ([Yuki Kishimoto])
* nostr: add `Event::get_tags_content` method ([Yuki Kishimoto])
* nostr: add `Event::hashtags` method ([Yuki Kishimoto])
* pool: allow to set event limits per kind ([Yuki Kishimoto])
* pool: log warn when high latency ([Yuki Kishimoto])
* sdk: add support to automatic authentication to relays (NIP-42) ([Yuki Kishimoto])
* ffi(nostr): add `Nip46Request` ([Yuki Kishimoto])
* ffi(sdk): add `NostrConnectRemoteSigner` ([Yuki Kishimoto])
* js(nostr): add missing NIP-57 functions ([Yuki Kishimoto])
* js(nostr): expose missing methods to `JsEvent` ([Yuki Kishimoto])
* book: add some python examples ([RydalWater])

### Fixed

* nostr: fix re-serialization of events that contains unknown keys during deserialization ([Yuki Kishimoto])
* nostr: fix `Nip21::to_nostr_uri` serialization ([Yuki Kishimoto])
* pool: fix relay doesn't auto reconnect in certain cases ([Yuki Kishimoto])
* nostr: add missing `TagStandard::PublicKeyLiveEvent` variant to `Event::public_keys` ([Yuki Kishimoto])
* sqlite: fix SQLite database panics when used outside the client context in bindings ([Yuki Kishimoto])
* sqlite: fix wrong event order when querying ([Yuki Kishimoto])

### Removed

* nostr: remove `verify_blocking` and `get_profile_blocking` functions ([Yuki Kishimoto])
* nostr: remove `RelayInformationDocument::get_blocking` ([Yuki Kishimoto])
* nostr: remove `blocking` feature ([Yuki Kishimoto])
* sqlite: removed `deadpool-sqlite` dep ([Yuki Kishimoto])
* ffi(nostr): remove `Keys::from_mnemonic_with_account` and `Keys::from_mnemonic_advanced` ([Yuki Kishimoto])

## [v0.31.0]

### Summary

Reworked `Tag`, added `TagStandard` enum, simplified the way to subscribe and/or reconcile to subset of relays
(respectively, `client.subscribe_to` and `client.reconcile_with`), added blacklist support to mute public keys or event IDs,
removed zap split from `client.zap` method, many improvements and more!

### Changed

* Bump `uniffi` to `v0.27.1` ([Yuki Kishimoto])
* nostr: update fingerprint of NIP26 functions ([Yuki Kishimoto])
* nostr: update fingerprint of `EventBuilder::zap_receipt` constructor ([Yuki Kishimoto])
* nostr: update `EventId::new` fingerprint ([Yuki Kishimoto])
* nostr: update fingerprint of `nip05::verify` function ([Yuki Kishimoto])
* nostr: improve performance of `Filter::match_event` ([Yuki Kishimoto])
* nostr: adj. kind to be `u16` instead of `u64` according to NIP01 ([Yuki Kishimoto])
* nostr: improve NIP19 serialization performance ([Yuki Kishimoto])
* nostr: improve `EventId::from_hex` performance ([Yuki Kishimoto])
* nostr: rename `Tag` enum to `TagStandard` ([Yuki Kishimoto])
* nostr: adj. NIP17 naming ([Yuki Kishimoto])
* nostr: allow to set a `Timestamp` tweak range ([Yuki Kishimoto])
* nostr: adj. NIP59 timestamp tweak range ([Yuki Kishimoto])
* nostr: reorganize `tag` module ([Yuki Kishimoto])
* nostr: manually impl `fmt::Debug` for `Publickey` ([Yuki Kishimoto])
* database: small improvements to flatbuffers `Event::encode` ([Yuki Kishimoto])
* ndb: bump `nostrdb` to `0.3.3` ([Yuki Kishimoto])
* rocksdb: bump `rocksdb` to `0.22` and set MSRV to `1.66.0` ([Yuki Kishimoto])
* pool: inline `RelayPool` methods ([Yuki Kishimoto])
* sdk: inline `Client`, `ClientBuilder` and `Options` methods ([Yuki Kishimoto])
* sdk: update `tokio` features ([Yuki Kishimoto])
* sdk: update visibility of `Options` field ([Yuki Kishimoto])
* sdk: remove zap split to support `rust-nostr` development from `Client::zap` method ([Yuki Kishimoto])
* signer: update fingerprint of `NostrConnectRemoteSigner::serve` method ([Yuki Kishimoto])
* ffi(nostr): set default args for `Nip19Profile` and `Nip19Event` constructors ([Yuki Kishimoto])
* ffi(nostr): set default args for `nip05::verify` function ([Yuki Kishimoto])
* ffi(sdk): set default args for `Client` constructors ([Yuki Kishimoto])
* js: enable support for Reference Types ([Yuki Kishimoto])
* js(nostr): rewrite `JsMetadata` methods and add getters ([Yuki Kishimoto])

### Added

* nostr: impl TryIntoUrl for &String ([Yuki Kishimoto])
* nostr: derive default traits for `HttpData`, `LiveEventHost` and `LiveEvent` ([Yuki Kishimoto])
* nostr: expose NIP49 `log_n` ([DanConwayDev])
* nostr: add tags indexes to `Event` ([Yuki Kishimoto])
* nostr: add `hex::decode_to_slice` ([Yuki Kishimoto])
* nostr: add `SecretKey::generate` ([Yuki Kishimoto])
* nostr: add `Tag` struct ([Yuki Kishimoto])
* nostr: add `EventBuilder::add_tags` method ([Yuki Kishimoto])
* database: add `author` index ([Yuki Kishimoto])
* pool: add `RelayPool::start` ([Yuki Kishimoto])
* pool: add `NegentropyDirection` default ([Yuki Kishimoto])
* sdk: add `Client::builder()` ([Yuki Kishimoto])
* sdk: add `Client::update_min_pow_difficulty` method ([Yuki Kishimoto])
* sdk: add `Client::connect_with_timeout` ([Yuki Kishimoto])
* sdk: add `Client::reconcile_with` and `Client::reconcile_advanced` ([Yuki Kishimoto])
* sdk: add `Client::subscribe_to` and `Client::subscribe_with_id_to` methods ([Yuki Kishimoto])
* sdk: add initial blacklist support ([Yuki Kishimoto])
* sdk: deprecate `Client::send_direct_msg` ([Yuki Kishimoto])
* ffi(nostr): add `gift_wrap_from_seal` func ([Yuki Kishimoto])
* js(nostr): add missing methods to `JsContact` ([Yuki Kishimoto])
* js(nostr): expose `util::generate_shared_key` ([Yuki Kishimoto])
* js(sdk): expose `Relay::subscribe` and `Relay::subscribe_with_id` methods ([Yuki Kishimoto])
* js(sdk): partially complete `JsRelay` ([Yuki Kishimoto])
* cli: add `sync` command ([Yuki Kishimoto])

### Fixed

* nostr: fix NIP19 event (`nevent`) serialization ([Yuki Kishimoto])

### Removed

* nostr: remove `GenericTagValue` ([Yuki Kishimoto])
* ffi(nostr): remove `Kind::match*` methods ([Yuki Kishimoto])

## [v0.30.0]

### Summary

Adapted NIP46 to last changes, added `NostrConnectRemoteSigner` to easily build remote signers (just construct it and call `serve` method),
improved proxy options (allow to specify the proxy target: all relays or only `.onion` ones),
improvements to NWC client, fixed equality operator for bindings (Python, Kotlin and Swift),
added `nostrdb` storage backend, added NIP32 and completed NIP51 support and more!

### Changed

* Bump `uniffi` to `v0.27` ([Yuki Kishimoto])
* Adapted NIP46 to last changes ([Yuki Kishimoto])
* nostr: change `Tag::parse` arg from `Vec<S>` to `&[S]` ([Yuki Kishimoto])
* nostr: allow to parse public key from NIP21 uri with `PublicKey::parse` ([Yuki Kishimoto])
* nostr: allow to parse event ID from NIP21 uri with `EventId::parse` ([Yuki Kishimoto])
* nostr: construct `GenericTagValue` based on `SingleLetterTag` in `deserialize_generic_tags` ([Yuki Kishimoto])
* nostr: set `UnsignedEvent` ID as optional ([Yuki Kishimoto])
* nostr: update `TryIntoUrl::try_into_url` fingerprint ([Yuki Kishimoto])
* nostr: bump `bitcoin` to `0.31` ([Yuki Kishimoto])
* sdk: bump `lnurl-pay` to `0.4` ([Yuki Kishimoto])
* sdk: improve `proxy` options ([Yuki Kishimoto])
* pool: bump `async-wsocket` to `0.4` ([Yuki Kishimoto])
* pool: return error if `urls` arg is empty in `InternalRelayPool::get_events_from` ([Yuki Kishimoto])
* pool: allow to disable `RelayLimits` ([Yuki Kishimoto])
* signer: re-work `nip46` module ([Yuki Kishimoto])
* nwc: avoid to open and close subscription for every request ([Yuki Kishimoto])
* nwc: allow to customize requests timeout ([Yuki Kishimoto])
* js(nostr): consume `JsEventBuilder` when building `Event` or `UnsignedEvent` ([Yuki Kishimoto])

### Added

* Add support to `nostrdb` storage backend ([Yuki Kishimoto])
* nostr: add `Report::Other` variant ([Daniel Cadenas])
* nostr: add `EventBuilder::reaction_extended` ([Yuki Kishimoto])
* nostr: add NIP32 support ([rustedmoon])
* pool: add `Relay::handle_notifications` ([Yuki Kishimoto])
* cli: add command to serve `Nostr Connect` signer ([Yuki Kishimoto])
* ffi(nostr): added `FilterRecord`, to allow to access fields in `Filter` ([Yuki Kishimoto])
* ffi(nostr): add missing NIP51 constructors ([rustedmoon])
* ffi(sdk): add `AbortHandle` ([Yuki Kishimoto])
* ffi(sdk): add `sqlite` and `ndb` features ([Yuki Kishimoto])
* js(nostr): add missing NIP51 constructors ([rustedmoon])
* js(nostr): add NIP47 request params and response results structs ([Yuki Kishimoto])
* js(sdk): add `NWC` client ([Yuki Kishimoto])
* js(sdk): add `NostrDatabase::save_event` method ([Xiao Yu])

### Fixed

* nostr: fix `Tag::content` return always `None` when `Tag::Generic` ([Yuki Kishimoto])
* nostr: fix NIP46 `Request::from_message` deserialization ([Yuki Kishimoto])
* nostr: fix `NostrConnectURI` serialization ([Yuki Kishimoto])
* nostr: fix `LookupInvoiceParams` ([benthecarman])
* ffi: fix equality operator (`==`) ([Yuki Kishimoto])
* js(nostr): fix `Keys` method calls in examples ([Xiao Yu])

### Removed

* Removed deprecated ([Yuki Kishimoto])

## v0.29.4

* pool: fix `InternalRelay::get_events_of_with_callback` timeout ([Yuki Kishimoto])

## v0.29.3

* pool: check filter limit in `InternalRelayPool::get_events_from` ([Yuki Kishimoto])

## v0.29.2

### Fixed

* pool: fix `get_events_of` issues ([Yuki Kishimoto])

## v0.29.1

### Fixed

* nostr: fix deserialization issues for events with non-standard `k` and `x` tags ([Yuki Kishimoto])
* pool: fix spurious send_event timeout error ([DanConwayDev] in https://github.com/rust-nostr/nostr/pull/375)

<!-- Contributors -->
[Yuki Kishimoto]: https://yukikishimoto.com
[DanConwayDev]: https://github.com/DanConwayDev
[Daniel Cadenas]: https://github.com/dcadenas
[rustedmoon]: https://github.com/rustedmoon
[benthecarman]: https://github.com/benthecarman
[Janek]: https://github.com/xeruf
[Xiao Yu]: https://github.com/kasugamirai
[RydalWater]: https://github.com/RydalWater
[lnbc1QWFyb24]: https://github.com/lnbc1QWFyb24
[reyamir]: https://github.com/reyamir
[w3irdrobot]: https://github.com/w3irdrobot
[nanikamado]: https://github.com/nanikamado
[rodant]: https://github.com/rodant
[erskingardner]: https://github.com/erskingardner

<!-- Tags -->
[Unreleased]: https://github.com/rust-nostr/nostr/compare/v0.36.0...HEAD
[v0.36.0]: https://github.com/rust-nostr/nostr/compare/v0.35.0...v0.36.0
[v0.35.0]: https://github.com/rust-nostr/nostr/compare/v0.34.0...v0.35.0
[v0.34.0]: https://github.com/rust-nostr/nostr/compare/v0.33.0...v0.34.0
[v0.33.0]: https://github.com/rust-nostr/nostr/compare/v0.32.0...v0.33.0
[v0.32.0]: https://github.com/rust-nostr/nostr/compare/v0.31.0...v0.32.0
[v0.31.0]: https://github.com/rust-nostr/nostr/compare/v0.30.0...v0.31.0
[v0.30.0]: https://github.com/rust-nostr/nostr/compare/v0.29.0...v0.30.0
