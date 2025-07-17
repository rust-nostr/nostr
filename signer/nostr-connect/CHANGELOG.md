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

- Remove `NostrConnect::get_relays` (https://github.com/rust-nostr/nostr/pull/894)

## v0.42.0 - 2025/05/20

### Breaking changes

- Encrypt NIP-46 events with NIP-44 instead of NIP-04 (https://github.com/rust-nostr/nostr/pull/862)
- Drop support for NIP-46 event decryption with NIP-04 (https://github.com/rust-nostr/nostr/pull/864)

## v0.41.0 - 2025/04/15

No notable changes in this release.

## v0.40.0 - 2025/03/18

No notable changes in this release.

## v0.39.0 - 2025/01/31

### Breaking changes

- Change `NostrConnect::shutdown` method signature

### Removed

- Remove `thiserror` dep
- Remove `async-trait` dep

## v0.38.0 - 2024/12/31

### Changed

- Require `fmt::Debug`, `Send` and `Sync` for `AuthUrlHandler`
- Improve secret matching for `NostrConnectRemoteSigner`
- Support both NIP04 and NIP44 for message decryption

### Added

- Add `NostrConnect::status`
- Add pubkey in `NostrConnectSignerActions::approve`

## v0.37.0 - 2024/11/27

### Breaking changes

- Refactor `NostrConnectRemoteSigner` to use distinct keys for signer and user
- Refactor `NostrConnectRemoteSigner` to use synchronous constructors

### Added

- Add `NostrConnect::non_secure_set_user_public_key`

## v0.36.0 - 2024/11/05

### Changed

- Rename `Nip46Signer` to `NostrConnect`
- Bootstrap NIP46 signer on demand

### Fixed

- Fix `NostrConnect` according to NIP46
