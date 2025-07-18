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

### Added

- Add automatic batching in ingester, write performance improvement
- Add `max_readers()` and `max_dbs()` configuration options to NostrLmdbBuilder
- Add NostrLmdbBuilder and allow setting a custom map size (https://github.com/rust-nostr/nostr/pull/970)

## v0.42.0 - 2025/05/20

No notable changes in this release.

## v0.41.0 - 2025/04/15

### Changed

- Enable POSIX semaphores for macOS and iOS targets (https://github.com/rust-nostr/nostr/commit/b58e0975f8ea53e794721a09d051b92c6a28212e)

## v0.40.0 - 2025/03/18

### Changed

- Bump MSRV to 1.72.0 (https://github.com/rust-nostr/nostr/pull/753)
- Implement event ingester (https://github.com/rust-nostr/nostr/pull/753)
- Avoid spawning thread for read methods (https://github.com/rust-nostr/nostr/pull/753)
- Avoid long-lived read txn when ingesting event (https://github.com/rust-nostr/nostr/pull/753)

## v0.39.0 - 2025/01/31

### Changed

- Use `EventBorrow` instead of `DatabaseEvent`

### Fixed

- Fix map size for 32-bit arch

## v0.38.0 - 2024/12/31

### Changed

- Use `async-utility` to spawn blocking tasks

### Removed

- Remove `thiserror` and `tracing` deps

## v0.37.0 - 2024/11/27

### Changed

- Optimize vector initialization in unit tests
- Commit also read txn
- Transactions improvements
- Improve NIP50 search performance

## v0.36.0 - 2024/11/05

### Changed

- Not save event deletion
- Return iterator instead of vector in `Lmdb::single_filter_query`
- Mark event as deleted only if database have the target event

### Fixed

- Add missing commit method call in `Store::delete`
- Fix unit tests
- Fix `Store::save_event` issues

## v0.35.0 - 2024/09/19

First release.
