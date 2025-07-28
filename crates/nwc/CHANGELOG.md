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

## v0.43.0 - 2025/07/28

### Added

- Add notification support for real-time payment updates (https://github.com/rust-nostr/nostr/pull/953)
- Add Monitor to NostrWalletConnectOptions (https://github.com/rust-nostr/nostr/pull/989)
- Add `NostrWalletConnectOptions::relay` method

### Deprecated

- Deprecate `NostrWalletConnectOptions::connection_mode` method

## v0.42.0 - 2025/05/20

No notable changes in this release.

## v0.41.0 - 2025/04/15

No notable changes in this release.

## v0.40.0 - 2025/03/18

### Changed

- Allow usage of multiple relays

## v0.39.0 - 2025/01/31

### Breaking changes

- Change `NWC::shutdown` method signature

## v0.38.0 - 2024/12/31

### Removed

- Remove `thiserror` dep and unnecessary `Error::Zapper` variant

## v0.37.0 - 2024/11/27

### Breaking changes

- Update `NWC::pay_invoice` method signature

### Changed

- Increase default timeout to 60 secs

### Added

- Add `NWC::status`

## v0.36.0 - 2024/11/05

No notable changes in this release.

## v0.35.0 - 2024/09/19

No notable changes in this release.

## v0.34.0 - 2024/08/15

### Changed

- Bump MSRV to v1.70.0

## v0.33.0 - 2024/07/16

No notable changes in this release.

## v0.32.0 - 2024/06/07

### Breaking changes

- Change `NWC::new` and `NWC::with_opts` fingerprint

## v0.31.0 - 2024/05/17

No notable changes in this release.

## v0.30.0 - 2024/04/15

### Changed

- Avoid opening and close subscription for every request
- Allow customizing requests timeout
