# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Summary

### Changed

* Adapted NIP46 to last changes ([Yuki Kishimoto])
* nostr: change `Tag::parse` arg from `Vec<S>` to `&[S]` ([Yuki Kishimoto])
* nostr: allow to parse public key from NIP21 uri with `PublicKey::parse` ([Yuki Kishimoto])
* nostr: allow to parse event ID from NIP21 uri with `EventId::parse` ([Yuki Kishimoto])
* nostr: construct `GenericTagValue` based on `SingleLetterTag` in `deserialize_generic_tags` ([Yuki Kishimoto])
* nostr: bump `bitcoin` to `0.31` ([Yuki Kishimoto])
* sdk: bump `lnurl-pay` to `0.4` ([Yuki Kishimoto])
* pool: bump `async-wsocket` to `0.4` ([Yuki Kishimoto])
* nwc: avoid to open and close subscription for every request ([Yuki Kishimoto])
* nwc: allow to customize requests timeout ([Yuki Kishimoto])
* js(nostr): consume `JsEventBuilder` when building `Event` or `UnsignedEvent` ([Yuki Kishimoto])

### Added

* ffi(nostr): added `FilterRecord`, to allow to access fields in `Filter` ([Yuki Kishimoto])
* ffi(sdk): add `AbortHandle` ([Yuki Kishimoto])

### Fixed

* nostr: fix `Tag::content` return always `None` when `Tag::Generic` ([Yuki Kishimoto])

### Removed

* Removed deprecated ([Yuki Kishimoto])

## v0.29.1

### Fixed

* pool: fix spurious send_event timeout error ([DanConwayDev] in https://github.com/rust-nostr/nostr/pull/375)

<!-- Contributors -->
[Yuki Kishimoto]: https://yukikishimoto.com
[DanConwayDev]: https://github.com/DanConwayDev

<!-- Tags -->
[Unreleased]: https://github.com/rust-nostr/nostr/compare/v0.29.0...HEAD
