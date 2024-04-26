# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Summary

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
* database: small improvements to flatbuffers `Event::encode` ([Yuki Kishimoto])
* pool: inline `RelayPool` methods ([Yuki Kishimoto])
* sdk: inline `Client`, `ClientBuilder` and `Options` methods ([Yuki Kishimoto])
* sdk: update `tokio` features ([Yuki Kishimoto])
* sdk: update visibility of `Options` field ([Yuki Kishimoto])
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
* pool: add `RelayPool::start` ([Yuki Kishimoto])
* pool: add `NegentropyDirection` default ([Yuki Kishimoto])
* sdk: add `Client::builder()` ([Yuki Kishimoto])
* sdk: add `Client::update_min_pow_difficulty` method ([Yuki Kishimoto])
* sdk: add `Client::connect_with_timeout` ([Yuki Kishimoto])
* sdk: add `Client::reconcile_with` and `Client::reconcile_advanced` ([Yuki Kishimoto])
* ffi(nostr): add `gift_wrap_from_seal` func ([Yuki Kishimoto])
* js(nostr): add missing methods to `JsContact` ([Yuki Kishimoto])
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

<!-- Tags -->
[Unreleased]: https://github.com/rust-nostr/nostr/compare/v0.30.0...HEAD
[v0.30.0]: https://github.com/rust-nostr/nostr/compare/v0.30.0...HEAD
