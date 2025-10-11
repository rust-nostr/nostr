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

- Add `RelayBuilder::max_filter_limit` and `RelayBuilder::default_filter_limit` to limit the filter's limit (https://github.com/rust-nostr/nostr/pull/1096)

### Fixed

- Consider a PoW difficulty if it’s greater than 0 in `RelayBuilder::min_pow` (https://github.com/rust-nostr/nostr/pull/1085)

## v0.43.0 - 2025/07/28

No notable changes in this release.

## v0.42.0 - 2025/05/20

### Added

- Add support for NIP-70 protected events (https://github.com/rust-nostr/nostr/pull/875)

## v0.41.0 - 2025/04/15

### Changed

- Send `CLOSED` if all possible events have been served (https://github.com/rust-nostr/nostr/pull/778)

## v0.40.0 - 2025/03/18

No notable changes in this release.

## v0.39.0 - 2025/01/31

### Changed

- Refactor shutdown mechanism to use `Notify` over `broadcast`
- Increase default max REQs to 500

### Added

- Custom http server

### Removed

- Remove `thiserror` dep
- Remove `async-trait` dep

## v0.38.0 - 2024/12/31

### Added

- Add NIP42 support
- Add negentropy support
- Add read/write policy plugins

## v0.37.0 - 2024/11/27

### Changed

- Port selection by using random port generation

### Added

- Add `RelayTestOptions`

## v0.36.0 - 2024/11/05

### Changed

- Refactor `Session::check_rate_limit` method
- Return error if event was deleted

### Added

- Add `LocalRelay` and `RelayBuilder`
- Allow to serve local relay as hidden service
- Allow to set number of max connections allowed
- Add `RelayBuilderMode`
- Add min POW difficulty option to `RelayBuilder`
- Handle ephemeral events

## v0.35.0 - 2024/09/19

First release.
