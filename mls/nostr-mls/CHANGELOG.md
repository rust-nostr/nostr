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

- Remove group type from groups (https://github.com/rust-nostr/nostr/commit/1deb718cf0a70c110537b505bdbad881d43d15cf)
- Removed `NostrMls::update_group_name`, `NostrMls::update_group_description`, `NostrMls::update_group_image` in favor of a single method for updating all group data.
- Added `admins` member to the `NostrGroupConfigData` (https://github.com/rust-nostr/nostr/pull/1050)

### Changed

- Upgrade openmls to v0.7.0 (https://github.com/rust-nostr/nostr/commit/b0616f4dca544b4076678255062b1133510f2813)
- Add `NostrMls::update_group_name`, `NostrMls::update_group_description`, `NostrMls::update_group_image` for updating the group data in the `NostrGroupDataExtension` (https://github.com/rust-nostr/nostr/commit/35d934d8ac8122f05e637bd9055e9e4a6167724a)

### Added

- Improved synchronization between MLSGroup and stored Group state on all commits. (https://github.com/rust-nostr/nostr/pull/1050)
- Added `NostrMls::update_group_data` method to handle updates of any of the fields of the `NostrGroupDataExtension` (https://github.com/rust-nostr/nostr/pull/1050)

## v0.43.0 - 2025/07/28

### Breaking changes

- Changed return type of `NostrMls::add_members` and `NostrMls::self_update` (https://github.com/rust-nostr/nostr/pull/934)
- Changed return type of all group and message methods to return Events instead of serialized MLS objects. (https://github.com/rust-nostr/nostr/pull/940)
- Changed the input params of `NostrMls::create_group`, and additional fields for `NostrGroupDataExtension` (https://github.com/rust-nostr/nostr/pull/965)

### Added

- Add `NostrMls::add_members` method for adding members to an existing group (https://github.com/rust-nostr/nostr/pull/931)
- Add `NostrMls::remove_members` method for removing members from an existing group (https://github.com/rust-nostr/nostr/pull/934)
- Add `NostrMls::leave_group` method for creating a proposal to leave the group (https://github.com/rust-nostr/nostr/pull/940)
- Add processing of commit messages and basic processing of proposals. (https://github.com/rust-nostr/nostr/pull/940)
- Add `ProcessedMessageState` for processed commits (https://github.com/rust-nostr/nostr/pull/954)
- Add method to check previous exporter_secrets when NIP-44 decrypting kind 445 messages (https://github.com/rust-nostr/nostr/pull/954)
- Add methods to update group name, description and image (https://github.com/rust-nostr/nostr/pull/978)

## v0.42.0 - 2025/05/20

First release (https://github.com/rust-nostr/nostr/pull/843)
