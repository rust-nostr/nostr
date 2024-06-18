# Changelog

<!-- All notable changes to this project will be documented in this file. -->

<!-- The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), -->
<!-- and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). -->

<!-- Template

## [Unreleased]

### Summary

### Changed

### Added

### Fixed

### Removed

-->

## [Unreleased]

### Summary

### Changed

* nostr: rename NIP-51 `EventBuilder` set constructors and `Kind` variants ([Yuki Kishimoto])
* nostr: small adj. to NIP-47 `ListTransactionsRequestParams` and `LookupInvoiceResponseResult` structs ([Yuki Kishimoto])
* nostr: add `identifier` arg to NIP-51 `EventBuilder` set constructors ([Yuki Kishimoto])
* pool: use per-purpose dedicated relay channels ([Yuki Kishimoto])
* pool: return relay urls to which `messages`/`events` have or not been sent for `send_*` and `batch_*` methods ([Yuki Kishimoto])
* ffi(sdk): convert `RelayPool::handle_notifications` method to async/future ([Yuki Kishimoto])
* js: increase max stack size to `0x1E84800` bytes (32 MiB) ([Yuki Kishimoto])

### Added

* nostr: add `EventBuilder::interest_set` ([Yuki Kishimoto])
* nostr: add `title`, `image` and `description` constructors to `Tag` ([Yuki Kishimoto])
* pool: add `SendOutput` and `SendEventOutput` structs ([Yuki Kishimoto])
* signer: add `NostrSigner::unwrap_gift_wrap` method ([Yuki Kishimoto])
* sdk: add `Client::unwrap_gift_wrap` method ([Yuki Kishimoto])
* js(sdk): partially expose `JsRelayPool` ([Yuki Kishimoto])
* book: add some python examples ([RydalWater])

### Fixed

* nostr: fix NIP-47 `list_transactions` response deserialization ([Yuki Kishimoto] and [lnbc1QWFyb24])
* pool: fix shutdown notification sent to external channel on `Relay::terminate` method call ([Yuki Kishimoto])
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
[Xiao Yu]: https://github.com/kasugamirai
[RydalWater]: https://github.com/RydalWater
[lnbc1QWFyb24]: https://github.com/lnbc1QWFyb24

<!-- Tags -->
[Unreleased]: https://github.com/rust-nostr/nostr/compare/v0.32.0...HEAD
[v0.32.0]: https://github.com/rust-nostr/nostr/compare/v0.31.0...v0.32.0
[v0.31.0]: https://github.com/rust-nostr/nostr/compare/v0.30.0...v0.31.0
[v0.30.0]: https://github.com/rust-nostr/nostr/compare/v0.29.0...v0.30.0
