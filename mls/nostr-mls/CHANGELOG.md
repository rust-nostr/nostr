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

- Changed return type of `NostrMls::add_members` and `NostrMls::self_update` (https://github.com/rust-nostr/nostr/pull/934)
- Changed return type of all group and message methods to return Events instead of serialized MLS objects. (https://github.com/rust-nostr/nostr/pull/940)
- Changed the input params of `NostrMls::create_group`, and additional fields for `NostrGroupDataExtension`. (https://github.com/rust-nostr/nostr/pull/965)

### Added

- Add `NostrMls::add_members` method for adding members to an existing group (https://github.com/rust-nostr/nostr/pull/931)
- Add `NostrMls::remove_members` method for removing members from an existing group (https://github.com/rust-nostr/nostr/pull/934)
- Add `NostrMls::leave_group` method for creating a proposal to leave the group (https://github.com/rust-nostr/nostr/pull/940)
- Add processing of commit messages and basic processing of proposals. (https://github.com/rust-nostr/nostr/pull/940)
- Add `ProcessedMessageState` for processed commits (https://github.com/rust-nostr/nostr/pull/954)
- Add method to check previous exporter_secrets when NIP-44 decrypting kind 445 messages (https://github.com/rust-nostr/nostr/pull/954)

## v0.42.0 - 2025/05/20

First release (https://github.com/rust-nostr/nostr/pull/843)
