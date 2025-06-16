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

- nostr: remove `NostrConnectMethod::GetRelays`, `NostrConnectRequest::GetRelays` and `ResponseResult::GetRelays` (https://github.com/rust-nostr/nostr/pull/894)
- nostr: remove `Market::Mention` (NIP-10) (https://github.com/rust-nostr/nostr/pull/895)
- nostr: remove `parser` feature (https://github.com/rust-nostr/nostr/pull/899)
- nostr: update `Nip19Profile::new` and `Nip19Coordinate::new` signature (https://github.com/rust-nostr/nostr/pull/910)
- nostr: update `RelayInformationDocument::get` signature (https://github.com/rust-nostr/nostr/pull/913)
- connect: remove `NostrConnect::get_relays` (https://github.com/rust-nostr/nostr/pull/894)
- database: merge traits into `NostrDatabase` (https://github.com/rust-nostr/nostr/pull/916)
- database: remove `NostrDatabase::has_coordinate_been_deleted` (https://github.com/rust-nostr/nostr/pull/917)

### Changed

- nostr: rework `NostrParser` (https://github.com/rust-nostr/nostr/pull/899)
- nostr: enhance `NostrParser` with flexible parsing options (https://github.com/rust-nostr/nostr/pull/912)
- nostr: impl `Any` for `NostrSigner` (https://github.com/rust-nostr/nostr/pull/918)
- database: impl `Any` for `NostrDatabase` (https://github.com/rust-nostr/nostr/pull/918)
- pool: refine notification sending depending on event database saving status (https://github.com/rust-nostr/nostr/pull/911)

### Added

- nostr: add NIP-88 support (https://github.com/rust-nostr/nostr/pull/892)
- nostr: add `Nip11GetOptions` (https://github.com/rust-nostr/nostr/pull/913)
- nostr: add `RelayUrl::domain` method (https://github.com/rust-nostr/nostr/pull/914)
- pool: allow putting relays to sleep when idle (https://github.com/rust-nostr/nostr/pull/926)

### Fixed

### Removed

- nostr: remove regex dep (https://github.com/rust-nostr/nostr/pull/899)

### Deprecated

- nostr: deprecate `nip21::extract_from_text` function (https://github.com/rust-nostr/nostr/pull/923)
- nostr: deprecate `Tags::from_text` constructor
- nostr: deprecate NIP-26 (https://github.com/rust-nostr/nostr/pull/928)

## v0.42.1 - 2025/05/26

### Added

- nostr: add detailed error handling for NIP-47 response deserialization (https://github.com/rust-nostr/nostr/pull/890)

### Fixed

- nostr: fix NIP-47 request params serialization (https://github.com/rust-nostr/nostr/pull/891)

## v0.42.0 - 2025/05/20

### Breaking changes

- nostr: rework nip46 module (https://github.com/rust-nostr/nostr/pull/865)
- pool: drop support for deprecated negentropy protocol (https://github.com/rust-nostr/nostr/pull/853)
- connect: encrypt NIP-46 events with NIP-44 instead of NIP-04 (https://github.com/rust-nostr/nostr/pull/862)
- connect: drop support for NIP-46 event decryption with NIP-04 (https://github.com/rust-nostr/nostr/pull/864)

### Changed

- Bump `lru` from 0.13.0 to 0.14.0
- nostr: rename `nip22::Comment` to `nip22::CommentTarget` (https://github.com/rust-nostr/nostr/pull/882)

### Added

- nostr: add `UnsignedEvent::id` method (https://github.com/rust-nostr/nostr/pull/868)
- nostr: add `TagKind::single_letter` constructor (https://github.com/rust-nostr/nostr/pull/871)
- nostr: add NIP-73 blockchain address and transaction (https://github.com/rust-nostr/nostr/pull/879)
- blossom: add new crate with Blossom support (https://github.com/rust-nostr/nostr/pull/838)
- mls-storage: add new crate with traits and types for mls storage implementations (https://github.com/rust-nostr/nostr/pull/836)
- mls-memory-storage: add an in-memory implementation for MLS (https://github.com/rust-nostr/nostr/pull/839)
- mls-sqlite-storage: a sqlite implementation for MLS (https://github.com/rust-nostr/nostr/pull/842)
- mls: add new crate for implementing MLS messaging (https://github.com/rust-nostr/nostr/pull/843)
- pool: add relay monitor (https://github.com/rust-nostr/nostr/pull/851)
- sdk: add `Options::pool`
- relay-builder: add support for NIP-70 protected events (https://github.com/rust-nostr/nostr/pull/875)

### Fixed

- nostr: handle `A` and `E` standard tags (https://github.com/rust-nostr/nostr/pull/870)
- nostr: fix `nip22::extract_root` to handle uppercase tags when `is_root` is true (https://github.com/rust-nostr/nostr/pull/876)
- pool: fix wrong conversion from u8 to RelayStatus (https://github.com/rust-nostr/nostr/pull/878)

### Deprecated

- sdk: deprecate `Options::notification_channel_size`

## v0.41.0 - 2025/04/15

### Breaking changes

- nostr: add optional relay URL arg to `Tag::coordinate`
- nostr: update `TagStandard::Label` and `EventBuilder::label`
- nostr: update `custom` field type in `Metadata` struct
- pool: remove `Error::Failed` variant
- pool: returns `Output` instead of an error if the message/event sending fails for all relays
- pool: add `reason` field to `AdmitStatus::Rejected` variant

### Changed

- lmdb: enable POSIX semaphores for macOS and iOS targets (https://github.com/rust-nostr/nostr/commit/b58e0975f8ea53e794721a09d051b92c6a28212e)
- ndb: bump nostrdb to 0.6.1
- pool: extend unit tests
- pool: better handling of `CLOSED` message for REQs (https://github.com/rust-nostr/nostr/pull/778)
- relay-builder: send `CLOSED` if all possible events have been served (https://github.com/rust-nostr/nostr/pull/778)

### Added

- nostr: add NIP-C0 (Code Snippets) support
- nostr: add `TagKind::u` constructor
- nostr: derive `Copy` for `HttpMethod`
- nostr: add `nip98::verify_auth_header`
- nostr: add `push`, `pop`, `insert` and `extend` methods to the `Tag` struct (https://github.com/rust-nostr/nostr/pull/817)
- nostr: add `nip47::Notification`
- nostr: add `MachineReadablePrefix::as_str` method
- nostr: derive `Hash` for `EventBuilder` and `Metadata`
- pool: add `Relay::ban` method
- pool: add `AdmitPolicy::admit_connection` method (https://github.com/rust-nostr/nostr/pull/831)
- keyring: add `NostrKeyring` (https://github.com/rust-nostr/nostr/pull/818)

### Fixed

- nostr: fix missing `transactions` object in serialization of nip47 ListTransactions ResponseResult
- nostr: fix NIP32 implementation (https://github.com/rust-nostr/nostr/commit/6979744839381ffa2b27f2d1efa5e13e522cdf24)

## v0.40.1 - 2025/03/24

### Fixed

- pool: fix `Relay::unsubscribe_all` method hangs

## v0.40.0 - 2025/03/18

### Breaking changes

- nostr: update `Nip19Event` relays field type from `Vec<String>` to `Vec<RelayUrl>`
- nostr: change the `Err` type of `ToBech32` to `Infallible` for `SecretKey`, `PublicKey` and `EventId`
- nostr: update `Tags::new` signature
- nostr: remove `WeakTag` (https://github.com/rust-nostr/nostr/pull/755)
- nostr: change `TagStandard::Relays` variant inner value from `Vec<Url>` to `Vec<RelayUrl>`
- nostr: split `NostrURI` into `ToNostrUri` and `FromNostrUri` traits
- nostr: replace generic parameter `AsRef<str>` with `&str` in `Coordinate::parse` and `Coordinate::from_kpi_format`
- nostr: replace generic parameter `AsRef<str>` with `&str` in `Nip21::parse`
- nostr: change `EventId::new` signature
- nostr: change `EventBuilder::git_repository_announcement` constructor signature
- nostr: change `EventBuilder::git_issue` constructor signature
- nostr: change `EventBuilder::git_patch` constructor signature
- nostr: `TagStandard::parse` now returns `Err(Error::UnknownStandardizedTag)` for non-lowercase hashtags as per NIP-24
- nostr: update `NostrWalletConnectURI` to support multiple relay URLs
- nostr: remove `EventIdOrCoordinate` enum
- nostr: change `EventBuilder::delete` arguments
- pool: drop `RelayFiltering`
- pool: remove `Relay` constructors
- pool: change `RelayPool::new` signature
- pool: now can set the notification channel size of a single `Relay` using `RelayOptions`
- sdk: change `Client::fetch_metadata` output
- sdk: remove `Client::state`

### Changed

- nostr: manually impl eq and cmp traits for `RelayUrl`
- nostr: use `Cow` in `ClientMessage` and `RelayMessage`
- nostr: derive `PartialOrd`, `Ord`, and `Hash` traits in `Nip21` enum
- pool: take event reference in `send_event` methods
- pool: use the relay ingester to perform actions
- pool: avoid spawning a task for every authentication request
- pool: use `std::sync::OnceLock` instead of `tokio::sync::OnceCell`
- lmdb: bump MSRV to 1.72.0 (https://github.com/rust-nostr/nostr/pull/753)
- lmdb: implement event ingester (https://github.com/rust-nostr/nostr/pull/753)
- lmdb: avoid spawning thread for read methods (https://github.com/rust-nostr/nostr/pull/753)
- lmdb: avoid long-lived read txn when ingesting event (https://github.com/rust-nostr/nostr/pull/753)
- ndb: return `None` in `NostrEventsDatabase::event_by_id` if event doesn't exist
- ndb: avoid event clone when calling `NostrEventsDatabase::save_event`
- pool: better handling of auto-closing subscription activity when fetching events (https://github.com/rust-nostr/nostr/pull/798)
- pool: reduce `WAIT_FOR_OK_TIMEOUT` to 10 secs
- pool: handle CLOSED message when syncing
- sdk: auto-update the gossip data when sending an event
- sdk: avoid full clone of relays when only urls are needed
- nwc: allow usage of multiple relays
- ffi: improve `Events::merge` and `Events::to_vec` performance
- ci: release wheels also for python `3.13`

### Added

- nostr: add NIP-38 support (https://github.com/rust-nostr/nostr/pull/771)
- nostr: add NIP-60 event kinds
- nostr: add NIP-62 support (https://github.com/rust-nostr/nostr/pull/777)
- nostr: add `NostrParser` (https://github.com/rust-nostr/nostr/pull/781)
- nostr: add `nip21::extract_from_text` function (https://github.com/rust-nostr/nostr/pull/754)
- nostr: add `EventBuilder::allow_self_tagging` (https://github.com/rust-nostr/nostr/pull/744)
- nostr: add `Nip19Event::from_event`
- nostr: add `Tag::client` constructor
- nostr: add `Tag::len` method (https://github.com/rust-nostr/nostr/pull/755)
- nostr: add `push`, `pop`, `insert`, `remove`, `extend` and `retain` methods to `Tags` struct (https://github.com/rust-nostr/nostr/pull/755)
- nostr: add `with_capacity`, `from_list`, `from_text` and `parse` constructors to `Tags` struct (https://github.com/rust-nostr/nostr/pull/755)
- nostr: add `Tags::dedup` method (https://github.com/rust-nostr/nostr/pull/755)
- nostr: add `EncryptedSecretKey::decrypt` method
- nostr: add `Nip19Coordinate` struct
- nostr: add `Coordinate::verify` method
- nostr: add `TagStandard::Client` variant
- nostr: add `EventBuilder::dedup_tags` method (https://github.com/rust-nostr/nostr/pull/772)
- nostr: impl `FromIterator<Tag>` for `Tags`
- nostr: add `EventDeletionRequest` struct
- nostr: add `notifications` field to NIP47 `GetInfoResponse`
- nostr: add `RelayMetadata::as_str` method
- nostr: add `nip42::is_valid_auth_event` function (https://github.com/rust-nostr/nostr/commit/e7a91ec69ab3b804cad0df8fccbcc53fd8dc7cc8)
- nostr: add `Tag::relays` constructor
- database: add `Events::force_insert`
- pool: event verification cache (https://github.com/rust-nostr/nostr/pull/746)
- pool: add `AdmitPolicy` trait (https://github.com/rust-nostr/nostr/pull/774)
- pool: add `ReqExitPolicy::WaitForEvents` variant (https://github.com/rust-nostr/nostr/pull/798)
- pool: add `RelayPoolBuilder`
- pool: add `RelayPool::is_shutdown` method
- ffi: add Mac Catalyst support in Swift package (https://github.com/rust-nostr/nostr/pull/749)
- js: add `KindStandard` enum

### Fixed

- nostr: fix `EventBuilder::git_repository_announcement` constructor according to last NIP34 rev (https://github.com/rust-nostr/nostr/pull/764)
- nostr: fix `EventBuilder::git_issue` constructor according to last NIP34 rev (https://github.com/rust-nostr/nostr/pull/764)
- nostr: fix `EventBuilder::git_patch` constructor according to last NIP34 rev (https://github.com/rust-nostr/nostr/pull/764)
- nostr: `Tag::hashtag` now lowercases the hashtag as per NIP-24

### Removed

- nostr: remove `RawRelayMessage`, `RawEvent`, `PartialEvent` and `MissingPartialEvent`
- database: remove deprecated
- pool: remove min POW difficulty check in favor of `AdmitPolicy` trait

### Deprecated

- Deprecate options to set min POW difficulty
- nostr: deprecate `EncryptedSecretKey::to_secret_key` method

## v0.39.0 - 2025/01/31

### Breaking changes

- Drop zapper support
- nostr: update `ClientMessage::neg_open` signature
- nostr: remove `ClientMessage::neg_open_deprecated`
- nostr: add `nip98` feature
- nostr: remove support for event JSON order preservation
- nostr: change `EventBuilder::gift_wrap` rumor arg type
- nostr: change `Filter::custom_tag` value arg type
- nostr: rename `Filter::remove_custom_tag` to `Filter::remove_custom_tags`
- nostr: take a single filter per REQ and COUNT
- nostr: rename `contact` module to `nip02`
- pool: change `Relay::connect` method signature
- pool: change `Relay::disconnect` method signature
- pool: change `RelayPool::disconnect` method signature
- pool: update `targets` arg type in `RelayPool::stream_events_targeted` method
- pool: update `RelayPool::remove_all_relays` method signature
- pool: update `RelayPool::force_remove_all_relays` method signature
- pool: update `RelayPool::shutdown` method signature
- sdk: update `Client::remove_all_relays` method signature
- sdk: update `Client::force_remove_all_relays` method signature
- sdk: update `Client::shutdown` method signature
- sdk: change `Client::disconnect` method signature
- sdk: change `Client::reset` method signature
- connect: change `NostrConnect::shutdown` method signature
- nwc: change `NWC::shutdown` method signature
- ffi: change `UnsignedEvent::tags` output
- ffi: convert `ImageDimensions` in a `Record`
- ffi: convert `Contact` to a `Record`
- ffi: rename `KindEnum` to `KindStandard`
- ffi: remove `KindStandard::Custom` variant
- ffi: rename `Kind::from_enum` and `Kind::as_enum` to `Kind::from_std` and `Kind::as_std`
- ffi: add `tor` feature (disabled by default)
- ffi: split `addr` in `ConnectionMode::Proxy` variant
- js: rename `Nip07Signer` to `BrowserSigner`
- bindings: remove `profile` module
- bindings: remove `NostrLibrary` struct and keep only `git_hash_version` func

### Changed

- Bump `async-wsocket` to 0.13
- Bump `negentropy` to 0.5
- nostr: refactor `PublicKey` to use byte array internally
- nostr: bump `bip39` to 2.1
- nostr: move `types::filter` to `crate::filter`
- nostr: move `Metadata` struct to `nip01` module
- nostr: cleanup error enums
- nostr: use `SECP256K1` global context from `secp256k1` crate
- nostr: manually implement `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` for `TagKind`
- nostr: derive `PartialEq` and `Eq` for event builder error
- nostr: rename `Nip07Signer` to `BrowserSigner`
- nostr: remove `#[cfg]` attributes from `NostrSigner` NIP04 and NIP44 methods
- nostr: omit everything after the relay-hint in NIP22 `p` tags
- nostr: make EventBuilder `custom_created_at` and `pow` fields public
- nostr: improve `Filter::match_event` performance
- pool: update `Error::WebSocket` variant inner type
- pool: refactor negentropy sync methods
- pool: refactor relay pinger
- pool: refactor relay disconnect logic
- pool: check if pool was already shutdown when calling `RelayPool::shutdown`
- pool: avoid repeatedly locking the relay channel receiver
- pool: refactor `RelayPool::stream_events_targeted`
- pool: refactor relay removal logic and add unit tests
- pool: handle `close` WebSocket message
- pool: always close WebSocket connection when handlers terminate
- pool: better control over the handling of the termination request
- pool: ensure notification subscription in subscribe auto-close logic
- lmdb: use `EventBorrow` instead of `DatabaseEvent`
- ndb: refactor note-to-event conversion
- relay-builder: refactor shutdown mechanism to use `Notify` over `broadcast`
- relay-builder: increase default max REQs to 500

### Added

- nostr: add NIP96 support
- nostr: add `Kind::PeerToPeerOrder` variant
- nostr: add `CowTag`
- nostr: add `EventBorrow`
- nostr: add `HttpData::to_authorization`
- nostr: add `CoordinateBorrow` struct
- nostr: add `Filter::custom_tags`
- nostr: add `nip22::extract_root` and `nip22:extract_parent`
- nostr: add `MachineReadablePrefix::Unsupported` variant
- database: add `Events::first_owned` and `Events::last_owned`
- database: impl `FlatBufferDecodeBorrowed` for `EventBorrow`
- database: add `NostrDatabaseWipe` trait
- pool: add `Relay::try_connect`
- pool: add `Relay::wait_for_connection`
- pool: add `RelayPool::try_connect`
- pool: add `RelayPool::try_connect_relay`
- pool: add `RelayPool::wait_for_connection`
- pool: add WebSocket transport abstraction
- sdk: add `Client::try_connect`
- sdk: add `Client::try_connect_relay`
- sdk: add `Client::wait_for_connection`
- sdk: add `ClientBuilder::websocket_transport`
- relay-builder: custom http server
- cli: allow setting a port in `serve` command

### Fixed

- lmdb: fix map size for 32-bit arch

### Removed

- nostr: remove `negentropy` deps
- nostr: remove `bitcoin` dep
- nostr: remove `once_cell` dep
- nostr: remove `async-trait` dep
- database: remove `async-trait` dep
- connect: remove `thiserror` dep
- connect: remove `async-trait` dep
- relay-builder: remove `thiserror` dep
- relay-builder: remove `async-trait` dep
- zapper: remove `async-trait` dep
- ffi: remove unnecessary `Arc`

### Deprecated

- nostr: deprecate `Keys::vanity`
- database: deprecate `NostrEventsDatabase::event_id_seen`
- database: deprecate `NostrEventsDatabase::event_seen_on_relays`
- sdk: deprecate `Options::req_filters_chunk_size`

## v0.38.0 - 2024/12/31

### Breaking changes

- nostr: update `FromBech32::from_bech32` method signature
- nostr: update `NostrURI::from_nostr_uri` method signature
- nostr: remove generics from parsing methods in `key` module
- nostr: remove generics from `EventId` parsing methods
- nostr: remove generic from `RelayUrl::parse`
- nostr: refactor `MachineReadablePrefix::parse` method to use `&str` directly
- nostr: update `RelayMessage::Notice` variant
- database: reduce default in-memory database limit to `35_000`
- database: update `NostrEventsDatabase::save_event` method signature
- pool: replace `Option<String>` with `String` in `Output::failed`
- sdk: update `fetch_*` and `stream_*` methods signature
- bindings: remove redundant parsing methods from `EventId`, `Coordinate`, `PublicKey` and `SecretKey`

### Changed

- Bump `async-utility` to 0.3, `async-wsocket` to 0.12 and `atomic-destructor` to 0.3
- nostr: remove self-tagging when building events
- nostr: don't set root tags when the root is null
- nostr: update `RelayMessage::NegErr` variant
- nostr: accept either `EventBuilder` or `UnsignedEvent` as rumor in NIP59 functions
- nostr: require `fmt::Debug`, `Send` and `Sync` for `NostrSigner`
- nostr: enable support for `Tags::indexes` in `no_std`
- nostr: improve `RelayMessage` docs
- nostr: include public key of root and parent author in `EventBuilder::comment`
- nostr: dedup tags in `EventBuilder::text_note_reply` and `EventBuilder::comment`
- nostr: don't use reply event as root `e` tag i no root is set in `EventBuilder::text_note_reply`
- database: add manual trait implementations for `BTreeCappedSet`
- database: replace LRU with custom memory cache for IDs tracking
- lmdb: use `async-utility` to spawn blocking tasks
- ndb: bump `nostr-ndb` to 0.5
- pool: add `PingTracker` and improve relay ping management
- pool: cleanup relay `Error` variants
- pool: acquire service watcher receiver outside the auto-connect loop
- pool: decrease `MAX_RETRY_INTERVAL` to 60 secs
- pool: rework retry interval calculation
- pool: improve shutdown docs
- pool: rename `FilterOptions` to `ReqExitPolicy`
- pool: log WebSocket connection error only if different from the last one
- pool: reduce atomic operations when cloning
- pool: derive `PartialOrd`, `Ord` and `Hash` for `RelayPoolNotification`
- sdk: refactor POW difficulty management
- connect: require `fmt::Debug`, `Send` and `Sync` for `AuthUrlHandler`
- connect: improve secret matching for `NostrConnectRemoteSigner`
- connect: support both NIP04 and NIP44 for message decryption
- zapper: bump `webln` to 0.4
- zapper: require `fmt::Debug`, `Send` and `Sync` for `NostrZapper`
- bindings: refactor `SendEventOutput` and `SubscribeOutput`

### Added

- nostr: add `Tags::challenge` method
- nostr: add `RelayUrl::is_local_addr`
- nostr: add `TagKind::k` constructor
- nostr: impl `IntoIterator` for `Tag`
- nostr: add NIP35 support
- nostr: add `Kind::is_addressable` and `ADDRESSABLE_RANGE`
- database: impl PartialEq and Eq for `Events`
- database: add `SaveEventStatus` enum
- pool: add `ReceiverStream`
- Add `SubscribeAutoCloseOptions::idle_timeout`
- sdk: automatically resend event after NIP-42 authentication
- sdk: add `Connection::embedded_tor_with_path`
- connect: add `NostrConnect::status`
- connect: add pubkey in `NostrConnectSignerActions::approve`
- relay-builder: add NIP42 support
- relay-builder: add negentropy support
- relay-builder: add read/write policy plugins

### Fixed

- nostr: remove redundant NIP10 tags from `EventBuilder::text_note_reply`
- sdk: fix NIP42 authentication for auto-closing REQ
- sdk: fix min POW is not updated to already existing relays
- bindings: allow passing empty string as relay url without return an error
- ffi: fix UniFFI checksum mismatch issue
- flutter: fix `default` is reserved in dart

### Removed

- nostr: remove `NegentropyErrorCode`
- database: remove `lru`, `thiserror` and `tracing` deps
- lmdb: remove `thiserror` and `tracing` deps
- indexeddb: remove `thiserror` and `tracing` deps
- zapper: remove `thiserror` dep
- pool: remove `thiserror` and `tokio-stream` deps
- pool: remove minimum interval constraint in `RelayOptions::retry_interval`
- sdk: remove `thiserror` and `nwc` deps
- nwc: remove `thiserror` dep and unnecessary `Error::Zapper` variant
- ffi: remove `MockRelay`
- ffi: remove `RawEvent` and `RawEventRecord`
- ffi: remove `NostrConnectRemoteSigner`
- bindings: remove `RelayPool`

### Deprecated

- nostr: deprecate `PARAMETERIZED_REPLACEABLE_RANGE`, `Kind::ParameterizedReplaceable` and `Kind::is_parameterized_replaceable`
- nostr: deprecate `JobRequest`, `JobResult`, `Regular`, `Replaceable` and `Ephemeral` kind variants
- pool: deprecated batch event methods
- pool: deprecate `FilterOptions`
- sdk: deprecate `timeout` option
- sdk: deprecate `Options::difficulty` and `Client::update_difficulty`

## v0.37.0 - 2024/11/27

### Breaking changes

- Use `RelayUrl` struct instead of `Url` for relay urls
- nostr: change `EventBuilder::gift_wrap` (and linked methods) args to take `extra_tags` instead of `expiration`
- nostr: change `EventBuilder::gift_wrap` (and linked methods) args to take an `EventBuilder` rumor instead of `UnsignedEvent`
- nostr: change `EventBuilder::private_msg_rumor` arg to take `extra_tags` instead of `reply_to`
- nostr: remove `tags` arg from `EventBuilder::new`
- nostr: remove `tags` arg from `EventBuilder::text_note`
- nostr: remove `tags` arg from `EventBuilder::long_form_text_note`
- nostr: remove `tags` arg from `EventBuilder::job_request`
- nostr: disable all default features except `std`
- nostr: change `Timestamp::to_human_datetime` method signature
- nostr: change `Tag::parse` arg from slice to iterator
- nostr: change `TagStandard::Relay` variant inner type
- nostr: remove `UncheckedUrl` struct
- nostr: update `NostrConnectURI::relays` to return slice
- nostr: update `NostrConnectURI::secret` to return string slice
- nostr: remove `-Params` and `-Result` suffix from NIP47 structs
- pool: switch from async to sync message sending for `Relay`
- connect: refactor `NostrConnectRemoteSigner` to use distinct keys for signer and user
- connect: refactor `NostrConnectRemoteSigner` to use synchronous constructors
- nwc: update `NWC::pay_invoice` method signature
- sdk: disable all default features
- sdk: set `Client::from_builder` as private
- ffi: convert `NostrSigner` trait to an object
- ffi: remove `NostrConnectURI::as_string`

### Changed

- nostr: rewrite `e` tag de/serialization
- pool: rework latency tracking
- pool: increase negentropy batch size down to 100
- pool: increase ping interval to 55 secs
- pool: increase max retry interval to 10 min
- pool: update retry interval calculation
- pool: try fetch relay information document only once every hour
- pool: not allow to add relays after `RelayPool` shutdown
- pool: rename `RelayOptions::retry_sec` to `RelayOptions::retry_interval`
- pool: rename `RelayOptions::adjust_retry_sec` to `RelayOptions::adjust_retry_interval`
- pool: request NIP11 document only after a successful WebSocket connection
- pool: immediately terminate relay connection on `Relay::disconnect` call
- pool: return error if relay doesn't exist when removing it
- sdk: cleanup `Client` methods
- sdk: fallback to READ relays if no relay list is set when breaking down filters
- relay-builder: port selection by using random port generation
- lmdb: optimize vector initialization in unit tests
- lmdb: commit also read txn
- lmdb: transactions improvements
- lmdb: improve NIP50 search performance
- nwc: increase default timeout to 60 secs

### Added

- nostr: add NIP104 tag and event kinds
- nostr: add `SingleLetterTag::as_str` and `TagKind::as_str`
- nostr: add `Kind::Comment`
- nostr: add `EventBuilder::comment`
- nostr: add uppercase field to `TagStandard::Coordinate` and `TagStandard::ExternalIdentity` variants
- nostr: add `TagStandard::Quote`
- nostr: add `Event::coordinate`
- nostr: add `A/a` tags in `EventBuilder::comment` (NIP22) events
- nostr: add NIP73 support
- nostr: add `NostrSigner::backend`
- nostr: add `EventBuilder::private_msg`
- nostr: add `EventBuilder::tag` and `EventBuilder::tags`
- nostr: add `nip17::extract_relay_list` and `nip17::extract_owned_relay_list`
- nostr: add `RelayUrl` struct
- database: add `NostrEventsDatabase` trait
- pool: add relay reconnection and disconnection unit tests
- pool: add `RelayServiceFlags::GOSSIP` flag
- sdk: allow to specify relay pool notification channel size in `Options`
- sdk: add support to NIP17 relay list
- relay-builder: add `RelayTestOptions`
- connect: add `NostrConnect::non_secure_set_user_public_key`
- nwc: add `NWC::status`
- ffi: add `make_private_msg` func
- ffi: add `CustomNostrSigner` trait
- ffi: impl `fmt::Display` for `NostrConnectURI`
- flutter: add `Tag` struct

### Fixed

- nostr: fix `TagStandard` de/serialization unit tests
- nostr: fix NIP90 kind ranges
- pool: fix relay can't manually connect if reconnection is disabled
- pool: fix reconnect loop not break if relay is disconnected while calling `Relay::disconnect`

### Removed

- nostr: remove `Marker::Custom` variant
- pool: remove `Relay::support_negentropy`
- pool: remove `Error::NotConnectedStatusChanged` variant
- pool: remove `INBOX` and `OUTBOX` flags
- ffi: remove `CustomNostrDatabase` trait

### Deprecated

- nostr: deprecate `EventBuilder::add_tags`
- pool: deprecate `RelayPoolNotification::RelayStatus` variant
- sdk: deprecate `Client::with_opts`
- sdk: deprecate `Options::connection_timeout`

## v0.36.0 - 2024/11/05

### Changed

- Bump toolchain channel to `1.82.0`
- Convert `nostr-signer` crate to `nostr-connect`
- nostr: move `TagsIndexes` into `Tags` struct
- nostr: use `OnceCell` implementation from `std` lib instead of `once_cell`
- nostr: remove redundant public key from repost events
- nostr: change `impl Ord for Event` behaviour (descending order instead of ascending)
- nostr: change `TagStandard::Relays` variant value from `Vec<UncheckedUrl>` to `Vec<Url>`
- nostr: reserve capacity for tags when POW is enabled in `EventBuilder`
- nostr: add `sign`, `sign_with_keys`, `sign_with_ctx`, `build` and `build_with_supplier` methods to `EventBuilder`
- nostr: deprecate `to_event`, `to_event_with_ctx` and `to_unsigned_event` methods
- relay-builder: refactor `Session::check_rate_limit` method
- relay-builder: return error if event was deleted
- pool: changes in `RelayPool::remove_relay` behavior
- pool: allow multi-filter reconciliation
- pool: increase negentropy frame size limit to `60_000`
- pool: set default max relay message size to 5MB
- pool: return error when receive `RelayNotification::Shutdown` variant
- pool: rename `NegentropyOptions` and `NegentropyDirection` to `SyncOptions` and `SyncDirection`
- pool: join futures instead of spawning threads in `RelayPool` methods
- pool: reduce overhead by maintaining only one atomic reference count for `RelayConnectionStats` and `RelayFiltering` structs
- pool: switch to atomic operations for `RelayStatus`
- pool: replace `RwLock` with `OnceCell` for `external_notification_sender`
- pool: convert `InternalRelay::send_notification` and linked methods to sync
- pool: avoid `RelayNotification` cloning when not needed in `InternalRelay::send_notification`
- pool: avoid full `InnerRelay` clone when requesting NIP11 document
- pool: rework relay connection methods and auto-connection logic
- pool: increase `MAX_ADJ_RETRY_SEC` to 120 secs
- pool: return reference instead of cloned structs for some getter methods of `Relay` and `RelayPool`
- pool: removed unnecessary timeout during the shutdown notification process
- pool: deprecate `RelaySendOptions::skip_disconnected`
- pool: deprecate `RelayConnectionStats::uptime`
- pool: better error for health check if relay status is `Initialized`
- pool: connect in chunks if too many relays
- pool: dynamic channel size for streaming of events
- pool: allow to define a limit of relays allowed in `RelayPool`
- pool: refactor `Relay::batch_event` and `Relay::auth`
- pool: deprecate `RelaySendOptions`
- sdk: deprecate `Client::get_events_of` and `Client::get_events_from` methods
- sdk: use `Events` instead of `Vec<Event>` in fetch and query methods
- sdk: rename `stream_events_of` to `stream_events`
- sdk: deprecate `Client::reconcile` and `Client::reconcile_with`
- sdk: use by default tor for onion relays if `tor` feature is enabled on non-mobile targets
- sdk: return reference to `RelayPool` instead of clone in `Client:pool`
- sdk: immediately return error if gossip filters are empty
- signer: auto enable `nip44` feature if `nip59` is enabled
- connect: rename `Nip46Signer` to `NostrConnect`
- database: improve `BTreeCappedSet`
- database: not save invalid event deletion
- lmdb: not save event deletion
- lmdb: return iterator instead of vector in `Lmdb::single_filter_query`
- lmdb: mark event as deleted only if database have the target event
- signer: bootstrap NIP46 signer on demand
- bindings(nostr): adj. `tag` module
- ffi: merge `nostr-ffi` in `nostr-sdk-ffi`
- js: merge `nostr-js` into `nostr-sdk-js`
- js: change `opt-level` to `z`

### Added

- nostr: add `TagKind::Client` variant
- nostr: add some shorthand constructors for `TagKind::SingleLetter`
- nostr: add `Tags` struct
- nostr: add `d` tag extraction test from `Tags`
- nostr: add `TagStandard::GitClone` and `TagKind::Clone` variants
- nostr: add `TagStandard::GitCommit` and `TagKind::Commit` variants
- nostr: add `TagStandard::GitEarliestUniqueCommitId` variant
- nostr: add `TagStandard::GitMaintainers` and `TagKind::Maintainers` variants
- nostr: add `TagStandard::Web` and `TagKind::Web` variants
- nostr: add `EventBuilder::git_repository_announcement`
- nostr: add `EventBuilder::git_issue`
- nostr: add `EventBuilder::git_patch`
- nostr: add `Tag::reference` constructor
- nostr: add `nip59::make_seal` function
- nostr: add `NostrSigner` trait
- database: add `Backend::is_persistent` method
- database: add `Events` struct
- relay-builder: add `LocalRelay` and `RelayBuilder`
- relay-builder: allow to serve local relay as hidden service
- relay-builder: allow to set number of max connections allowed
- relay-builder: add `RelayBuilderMode`
- relay-builder: add min POW difficulty option to `RelayBuilder`
- relay-builder: handle ephemeral events
- pool: add `RelayPool::force_remove_relay` method
- pool: add `RelayFiltering::overwrite_public_keys` method
- pool: add `RelayPool::sync_targeted`
- pool: add `Relay::reconcile_multi`
- pool: negentropy sync progress
- pool: add `RelayConnectionStats::success_rate`
- sdk: add `Client::fetch_events` and `Client::fetch_events_from` methods
- sdk: add `Client::sync` and `Client::sync_with` methods
- sdk: add gossip support to `Client::sync`
- sdk: add `Client::force_remove_all_relays`
- sdk: add `Client::reset` and `switch-account` example
- signer: add `NostrSigner::gift_wrap`
- zapper: add `WebLNZapper` struct (moved from `nostr-webln` crate)
- ffi(nostr): add `tag_kind_to_string` func
- ffi(nostr): add `Tag::kind_str` method
- ffi(nostr): impl `Display` for `Kind`
- js(nostr): add `JsKind::_to_string` method
- js(nostr): expose `from_nostr_uri` and `to_nostr_uri` for `PublicKey` and `EventId`
- cli: show negentropy sync progress

### Fixed

- nostr: adj. `NostrConnectURI` de/serialization according to NIP46
- connect: fix `NostrConnect` according to NIP46
- lmdb: add missing commit method call in `Store::delete`
- lmdb: fix unit tests
- lmdb: fix `Store::save_event` issues
- sdk: fix `filters empty` error when gossip option is enabled

### Removed

- Remove deprecated
- pool: remove `RelayPool::reconcile_advanced`
- pool: remove `RelayPool::reconcile_with_items`
- webln: remove `nostr-webln` crate
- sqlite: remove `nostr-sqlite` crate

## v0.35.0 - 2024/09/19

### Changed

- nostr: bump `bitcoin` to `v0.32`
- nostr: bump `base64` to `v0.22`
- nostr: deprecate `Event::from_value`
- nostr: deprecate `Tag::as_vec`
- nostr: re-write `RawRelayMessage` parsing
- nostr: update `Event` fields
- nostr: deprecate `Event::is_*` kind related methods
- nostr: change `TryIntoUrl::Err` to `Infallible` for `Url`
- nostr: change `Event::verify_id` and `Event::verify_signature` fingerprint
- nostr: impl custom `Debug`, `PartialEq` and `Eq` for `Keys`
- nostr: impl `PartialOrd`, `Ord` and `Hash` for `Keys`
- nostr: change `Keys::secret_key` and `Keys::sign_schnorr` methods fingerprint
- nostr: deprecate `Keys::generate_without_keypair`
- nostr: change NIP26 functions fingerprint
- nostr: improve `NostrWalletConnectURI` parsing
- nostr: update `EventBuilder::job_feedback` method fingerprint
- nostr: deprecate `EventBuilder::to_pow_event`
- nostr: impl `Display` for `MachineReadablePrefix`
- nostr: improve `Keys` docs
- nostr: change visibility of `public_key` field in `Keys` struct
- nostr: deprecate `Keys::public_key_ref`
- nostr: use `OsRng` instead of `ThreadRng` for `SECP256K1` global context and schnorr signing
- nostr: improve `Timestamp::to_human_datetime` performance
- nostr: deprecate `EventId::owned`
- nostr: convert `EventId::all_zeroes` to const function
- nostr: convert `Timestamp::from_secs` to const function
- nostr: deprecate `Kind::as_u32` and `Kind::as_u64`
- database: update `NostrDatabase` supertraits
- database: impl `Clone` for `MemoryDatabase`
- database: update `NostrDatabase::event_by_id` fingerprint
- relay-builder: bump `tokio-tungstenite` to `v0.24`
- pool: bump `async-wsocket` to `v0.8`
- pool: avoid unnecessary `Url` and `Relay` clone in `RelayPool` methods
- pool: avoid `Relay` clone in `RelayPool::connect_relay` method
- pool: `RelayPool::send_event` and `RelayPool::batch_event` send only to relays with `WRITE` flag
- pool: `RelayPool::subscribe_with_id`, `RelayPool::get_events_of` and `RelayPool::stream_events_of` REQ events only to relays with `READ` flag
- pool: bump `async-wsocket` to `v0.9`
- pool: improve `Relay::support_negentropy` method
- pool: change handle relay message log level from `error` to `warn`
- signer: update NIP04 and NIP44 methods signature
- webln: bump `webln` to `v0.3`
- sqlite: deprecate `SQLiteDatabase` in favor of LMDB
- sdk: bump `lnurl-pay` to `v0.6`
- sdk: update `Client::gift_wrap` and `Client::gift_wrap_to` methods signature
- sdk: document and rename `Client::metadata` to `Client::fetch_metadata`
- sdk: update `Client::shutdown` method fingerprint
- sdk: deprecate `Client::add_relay_with_opts` and `Client::add_relays`
- sdk: deprecate `RelayPool::send_msg` and `RelayPool::batch_msg`
- sdk: inherit pool subscriptions only when calling `Client::add_relay` or `Client::add_read_relay` methods
- ffi(nostr): impl `Display` for `Coordinate`
- ffi(sdk): change `Connection::embedded_tor` fingerprint for `android` and `ios` targets
- cli: rename `open` command to `shell`
- cli: rename `serve-signer` command to `bunker`

### Added

- nostr: impl `TryFrom<Vec<Tag>>` for `LiveEvent`
- nostr: add `Tag::as_slice`
- nostr: add `NostrWalletConnectURI::parse`
- nostr: add `JobFeedbackData` struct
- nostr: add `EventBuilder::pow` method
- nostr: add `TagKind::custom` constructor
- nostr: add `Timestamp::from_secs`
- nostr: add `EventId::from_byte_array`
- nostr: add `Timestamp::min` and `Timestamp::max`
- nostr: add `nip65::extract_owned_relay_list`
- nostr: add `Kind::from_u16`
- database: add `DatabaseHelper::fast_query`
- database: add `NostrDatabase::check_id`
- database: add `NostrDatabaseExt::relay_lists`
- lmdb: add LMDB storage backend
- relay-builder: add `MockRelay`
- pool: add `RelayPool::disconnect_relay` method
- pool: add `RelayPool::relays_with_flag` and `RelayPool::all_relays`
- pool: add support to negentropy v1
- pool: add whitelist support
- sdk: add `Client::add_discovery_relay`
- sdk: add `Client::add_read_relay` and `Client::add_write_relay`
- sdk: add `Client::stream_events_targeted`
- sdk: add `Client::subscribe_targeted`
- sdk: add gossip support to `Client::send_event`
- sdk: add gossip support to `Client::get_events_of` and `Client::stream_events_of`
- sdk: add gossip support to `Client::subscribe` and `Client::subscribe_with_id`
- bindings(nostr): expose `as_pretty_json` for some structs
- bindings(sdk): expose `Client::fetch_metadata`
- bindings(sdk): expose `Client::pool` method
- ffi(nostr): expose `Kind::is_*` methods
- ffi(sdk): expose `MockRelay`
- js(nostr): add `Kind` object
- js(nostr): expose `getNip05Profile` function
- js(nostr): expose missing methods to `JsCoordinate`
- js(sdk): expose `RelayPool::relays`
- cli: add `serve` command
- cli: add shell history

### Fixed

- nostr: fix `TagStanderd::to_vec`
- nostr: fix broken intra doc links
- nostr: fix `JsonUtil::try_as_pretty_json` method
- nostr: fix `Kind::is_regular` method

### Removed

- Drop support for `rocksdb`
- nostr: remove `bech32` from the public API
- nostr: remove `Keys::from_public_key`
- nostr: remove `tracing` dep
- nostr: remove impl `fmt::Display` for `SecretKey`
- database: remove `has_event_already_been_saved`, `has_event_already_been_seen` and `has_event_id_been_deleted` methods from `NostrDatabase`
- database: remove `Err` from `NostrDatabase`
- database: remove `NostrDatabase::bulk_import`
- database: remove `DatabaseError::NotFound` variant
- database: remove `DatabaseError::Nostr` variant
- database: remove `Order` enum
- database: remove `order` arg from `NostrDatabase::query`
- pool: remove high latency log
- pool: remove `Error::OneShotRecvError` variant
- zapper: remove `Err` from `NostrZapper` and unnecessary variants from `ZapperError`
- js(nostr): remove `Keys::vanity`
- cli: remove `reverse` flag from `query` command

## v0.34.0 - 2024/08/15

### Changed

- Bump MSRV to v1.70.0
- Bump toolchain channel to `1.80.1`
- nostr: deprecate `Event::author_ref` and `Event::iter_tags`
- nostr: calculate `EventId` in `EventBuilder::to_unsigned_event_with_supplier`
- nostr: ensure that NIP59 rumor has `EventId`
- nostr: update `PartialEvent` methods
- nostr: change `EventBuilder::award_badge` fingerprint
- nostr: add NIP50 support to `Filter::match_event` method
- nostr: remove `Arc<T>` from `OnceCell<T>` in `Event` and `Tag`
- nostr: move `sig` field from `PartialEvent` to `MissingPartialEvent`
- nostr: better `Debug` trait impl for `EventId`, `PublicKey` and `Tag`
- nostr: improve `SubscriptionId::generate_with_rng`
- pool: take mutex ownership instead of clone in `InternalRelayPool::get_events_from`
- pool: remove IDs collection from `InternalRelayPool::get_events_from`
- pool: better checks before perform queries or send messages to relays
- pool: bump `async-wsocket` to `v0.7`
- pool: get events only from remote relay when calling `get_events_of` or `get_events_from`
- database: avoid to copy `EventId` in `Event::decode`
- database: use `Vec` instead of `BTreeSet` as inner value for `TagIndexValues`
- database: rework `DatabaseIndexes` and rename to `DatabaseHelper`
- database: allow to set max capacity to `DatabaseHelper`
- database: speedup helper bulk load
- database: set a default logic for `NostrDatabase::negentropy_items`
- sdk: rename `Proxy` and `ProxyTarget` to `Connection` and `ConnectionTarget`
- sdk: allow to skip slow relays
- sdk: allow to specify the source of events for `Client::get_events_of` method
- sdk: deprecate `Client::get_events_of_with_opts`
- sqlite: use `ValueRef` instead of owned one
- cli: improve `sync` command
- cli: allow to specify relays in `open` command

### Added

- nostr: add NIP31 support
- nostr: add NIP70 support
- nostr: add `EventId::LEN` const
- nostr: add `UnsignedEvent::ensure_id` method
- nostr: add missing `payload` arg to `EventBuilder::job_result`
- nostr: add `ConversationKey::new`
- nostr: add `Request::multi_pay_invoice` constructor
- nostr: add `Jsonutil::as_pretty_json` and `JsonUtil::try_as_pretty_json` methods
- nostr: add `Coordinate::has_identifier`
- pool: add `RelayPoolNotification::Authenticated` variant
- pool: add `RelayPool::save_subscription`
- sqlite/rocksdb/indexeddb: allow to open database with limited capacity
- sdk: add `Client::gift_wrap_to` and `Client::send_private_msg_to`
- sdk: add option to autoconnect relay on `Client::add_relay` method call
- sdk: add support to embedded tor client
- sdk: add `Options::max_avg_latency`
- sdk: add `Client::stream_events_of` and `Client::stream_events_from` methods
- ffi(nostr): add `EventBuilder::seal` constructor
- cli: add `generate` command
- cli: add `json` flag to `query` command

### Fixed

- pool: fix `Event` notification variant sent also for events sent by the SDK
- database: fix indexes `QueryPattern`
- database: fix query issue due to wrong tag value order

### Removed

- Remove deprecated methods/functions
- nostr: remove support for `nrelay` NIP19 entity
- nostr: remove support for NIP44 v1
- nostr: remove `EventBuilder::encrypted_direct_msg`
- database: remove `TempEvent`
- database: remove `NostrDatabase::event_ids_by_filters`
- sdk: remove `Client::send_direct_msg`
- cli: remove `tracing-subscriber` dep

## v0.33.0 - 2024/07/16

### Changed

- Bump `uniffi` to `v0.28.0`
- nostr: rename NIP-51 `EventBuilder` set constructors and `Kind` variants
- nostr: small adj. to NIP-47 `ListTransactionsRequestParams` and `LookupInvoiceResponseResult` structs
- nostr: add `identifier` arg to NIP-51 `EventBuilder` set constructors
- nostr: change `nip65::extract_relay_list` fingerprint
- nostr: avoid allocation where possible in NIP-05 module
- nostr: get NIP-46 relays from NIP-05 address
- nostr: deprecate `EventBuilder::encrypted_direct_msg`
- pool: use per-purpose dedicated relay channels
- pool: return relay urls to which `messages`/`events` have or not been sent for `send_*` and `batch_*` methods
- pool: return relay urls to which `subscription` have or not been success for `subscribe*` methods
- pool: rename `Relay::terminate` to `Relay::disconnect`
- pool: always send `RelayPoolNotification::Message` variant
- pool: return report for negentropy reconciliation
- signer: use `limit(0)` instead of `since` for `Nip46Signer` subscription filter
- signer: deprecate `NostrConnectRemoteSigner::nostr_connect_uri` and `Nip46Signer::nostr_connect_uri`
- sdk: allow to change auto authentication to relays option (NIP-42) after client initialization
- sdk: retrieve contact list public keys only from the latest events
- sdk: re-subscribe closed subscriptions after NIP-42 authentication
- bindings(nostr): allow to specify coordinates in `EventBuilder::delete` constructor
- ffi(sdk): convert `RelayPool::handle_notifications` method to async/future
- js: increase max stack size to `0x1E84800` bytes (32 MiB)
- js(nostr): adj. method names to camelcase format

### Added

- nostr: add `EventBuilder::interest_set`
- nostr: add `title`, `image` and `description` constructors to `Tag`
- nostr: add `Timestamp::zero` and `Timestamp::is_zero` methods
- nostr: add `Nip05Profile` struct
- nostr: add `nip05::profile` function
- nostr: add `LEN` const to `PublicKey`, `SecretKey` and `EncryptedSecretKey`
- nostr: add `Report::Malware` variant
- nostr: add `coordinate` methods to `Filter` struct
- nostr: add NIP-34 kinds
- nostr: add `MachineReadablePrefix` enum
- nostr: add `ClientMessage::is_auth`
- pool: add `Output<T>` struct
- pool: add `Output<EventId>::id` and `Output<SubscriptionId>::id` methods
- pool: add dry run option for negentropy reconciliation
- signer: add `NostrSigner::unwrap_gift_wrap` method
- signer: add `bunker_uri` method to NIP-46 client and signer
- sdk: add `Client::unwrap_gift_wrap` method
- js(nostr): complete `JsFilter` struct
- js(sdk): partially expose `JsRelayPool`

### Fixed

- nostr: fix NIP-47 `list_transactions` response deserialization
- pool: fix shutdown notification sent to external channel on `Relay::terminate` method call
- pool: fix `RelayPool::reconcile_advanced` method uses database items instead of the passed ones
- signer: add missing NIP-46 connect "ACK" message handling
- sdk: fix NIP-42 client authentication
- js: fix "RuntimeError: memory access out of bounds" WASM error

### Removed

- pool: remove `RelayPoolNotification::Stop`
- pool: remove `RelayStatus::Stop`
- Remove all `start` and `stop` methods

## v0.32.0 - 2024/06/07

### Changed

- Bump `atomic-destructor` to `v0.2`
- Bump `uniffi` to `v0.27.2`
- nostr: ignore malformed public keys during NIP19 event (`nevent`) parsing
- nostr: update `Event::pubic_keys` and `Event_event_ids` methods
- nostr: adj. NIP-10 support
- nostr: change fingerprint of `nip05::verify`
- nostr: rework `TagStandard::parse`
- nostr: add `a` tag to zap receipts
- nostr: change NIP-07 `Error::Wasm` variant value from `JsValue` to `String`
- nostr: update `EventBuilder::live_event_msg` fingerprint
- nostr: set `kind` arg in `EventBuilder::reaction_extended` as optional
- pool: increase default kind 3 event limit to `840000` bytes and `10000` tags
- pool: improve accuracy of latency calculation
- pool: refactoring and adj. `relay` internal module
- pool: log when websocket messages are successfully sent
- pool: always close the WebSocket when receiver loop is terminated
- pool: use timeout for WebSocket message sender
- pool: bump `async-wsocket` to `v0.5`
- sdk: send NIP-42 event only to target relay
- sqlite: bump `rusqlite` to `v0.31`
- nwc: change `NWC::new` and `NWC::with_opts` fingerprint
- ffi: migrate kotlin packages to `org.rust-nostr`
- bindings(sdk): log git hash after logger initialization
- ffi(nostr): set default args values where possible
- ffi(nostr): convert `verify_nip05` and `get_nip05_profile` to async functions
- ffi(nostr): convert `RelayInformationDocument::get` to async
- ffi(nostr): merge `Keys::from_mnemonic_*` constructors into `Keys::from_menmonic`
- ffi(sdk): add `async/future` support (convert from blocking to async)
- ffi(sdk): no longer spawn a thread when calling `handle_notifications`
- js(sdk): change `JsNostrZapper::nwc` fingerprint
- js(sdk): rename `JsNip46Signer::new` to `JsNip46Signer::init`
- ci: build python wheels for `manylinux_2_28_x86_64`

### Added

- nostr: add `Tag::is_root` method
- nostr: add `JsonUtil::try_as_json` method
- nostr: add `public_key` field to `TagStandard::Event`
- nostr: add support to `nrelay` NIP-19 entity
- nostr: add `Event::get_tag_content` method
- nostr: add `Event::get_tags_content` method
- nostr: add `Event::hashtags` method
- pool: allow to set event limits per kind
- pool: log warn when high latency
- sdk: add support to automatic authentication to relays (NIP-42)
- ffi(nostr): add `Nip46Request`
- ffi(sdk): add `NostrConnectRemoteSigner`
- js(nostr): add missing NIP-57 functions
- js(nostr): expose missing methods to `JsEvent`

### Fixed

- nostr: fix re-serialization of events that contains unknown keys during deserialization
- nostr: fix `Nip21::to_nostr_uri` serialization
- pool: fix relay doesn't auto reconnect in certain cases
- nostr: add missing `TagStandard::PublicKeyLiveEvent` variant to `Event::public_keys`
- sqlite: fix SQLite database panics when used outside the client context in bindings
- sqlite: fix wrong event order when querying

### Removed

- nostr: remove `verify_blocking` and `get_profile_blocking` functions
- nostr: remove `RelayInformationDocument::get_blocking`
- nostr: remove `blocking` feature
- sqlite: removed `deadpool-sqlite` dep
- ffi(nostr): remove `Keys::from_mnemonic_with_account` and `Keys::from_mnemonic_advanced`

## v0.31.0 - 2024/05/17

### Changed

- Bump `uniffi` to `v0.27.1`
- nostr: update fingerprint of NIP26 functions
- nostr: update fingerprint of `EventBuilder::zap_receipt` constructor
- nostr: update `EventId::new` fingerprint
- nostr: update fingerprint of `nip05::verify` function
- nostr: improve performance of `Filter::match_event`
- nostr: adj. kind to be `u16` instead of `u64` according to NIP01
- nostr: improve NIP19 serialization performance
- nostr: improve `EventId::from_hex` performance
- nostr: rename `Tag` enum to `TagStandard`
- nostr: adj. NIP17 naming
- nostr: allow to set a `Timestamp` tweak range
- nostr: adj. NIP59 timestamp tweak range
- nostr: reorganize `tag` module
- nostr: manually impl `fmt::Debug` for `Publickey`
- database: small improvements to flatbuffers `Event::encode`
- ndb: bump `nostrdb` to `0.3.3`
- rocksdb: bump `rocksdb` to `0.22` and set MSRV to `1.66.0`
- pool: inline `RelayPool` methods
- sdk: inline `Client`, `ClientBuilder` and `Options` methods
- sdk: update `tokio` features
- sdk: update visibility of `Options` field
- sdk: remove zap split to support `rust-nostr` development from `Client::zap` method
- signer: update fingerprint of `NostrConnectRemoteSigner::serve` method
- ffi(nostr): set default args for `Nip19Profile` and `Nip19Event` constructors
- ffi(nostr): set default args for `nip05::verify` function
- ffi(sdk): set default args for `Client` constructors
- js: enable support for Reference Types
- js(nostr): rewrite `JsMetadata` methods and add getters

### Added

- nostr: impl TryIntoUrl for &String
- nostr: derive default traits for `HttpData`, `LiveEventHost` and `LiveEvent`
- nostr: expose NIP49 `log_n`
- nostr: add tags indexes to `Event`
- nostr: add `hex::decode_to_slice`
- nostr: add `SecretKey::generate`
- nostr: add `Tag` struct
- nostr: add `EventBuilder::add_tags` method
- database: add `author` index
- pool: add `RelayPool::start`
- pool: add `NegentropyDirection` default
- sdk: add `Client::builder()`
- sdk: add `Client::update_min_pow_difficulty` method
- sdk: add `Client::connect_with_timeout`
- sdk: add `Client::reconcile_with` and `Client::reconcile_advanced`
- sdk: add `Client::subscribe_to` and `Client::subscribe_with_id_to` methods
- sdk: add initial blacklist support
- sdk: deprecate `Client::send_direct_msg`
- ffi(nostr): add `gift_wrap_from_seal` func
- js(nostr): add missing methods to `JsContact`
- js(nostr): expose `util::generate_shared_key`
- js(sdk): expose `Relay::subscribe` and `Relay::subscribe_with_id` methods
- js(sdk): partially complete `JsRelay`
- cli: add `sync` command

### Fixed

- nostr: fix NIP19 event (`nevent`) serialization

### Removed

- nostr: remove `GenericTagValue`
- ffi(nostr): remove `Kind::match*` methods

## v0.30.0 - 2024/04/15

### Changed

- Bump `uniffi` to `v0.27`
- Adapted NIP46 to last changes
- nostr: change `Tag::parse` arg from `Vec<S>` to `&[S]`
- nostr: allow to parse public key from NIP21 uri with `PublicKey::parse`
- nostr: allow to parse event ID from NIP21 uri with `EventId::parse`
- nostr: construct `GenericTagValue` based on `SingleLetterTag` in `deserialize_generic_tags`
- nostr: set `UnsignedEvent` ID as optional
- nostr: update `TryIntoUrl::try_into_url` fingerprint
- nostr: bump `bitcoin` to `0.31`
- sdk: bump `lnurl-pay` to `0.4`
- sdk: improve `proxy` options
- pool: bump `async-wsocket` to `0.4`
- pool: return error if `urls` arg is empty in `InternalRelayPool::get_events_from`
- pool: allow to disable `RelayLimits`
- signer: re-work `nip46` module
- nwc: avoid to open and close subscription for every request
- nwc: allow to customize requests timeout
- js(nostr): consume `JsEventBuilder` when building `Event` or `UnsignedEvent`

### Added

- Add support to `nostrdb` storage backend
- nostr: add `Report::Other` variant
- nostr: add `EventBuilder::reaction_extended`
- nostr: add NIP32 support
- pool: add `Relay::handle_notifications`
- cli: add command to serve `Nostr Connect` signer
- ffi(nostr): added `FilterRecord`, to allow to access fields in `Filter`
- ffi(nostr): add missing NIP51 constructors
- ffi(sdk): add `AbortHandle`
- ffi(sdk): add `sqlite` and `ndb` features
- js(nostr): add missing NIP51 constructors
- js(nostr): add NIP47 request params and response results structs
- js(sdk): add `NWC` client
- js(sdk): add `NostrDatabase::save_event` method

### Fixed

- nostr: fix `Tag::content` return always `None` when `Tag::Generic`
- nostr: fix NIP46 `Request::from_message` deserialization
- nostr: fix `NostrConnectURI` serialization
- nostr: fix `LookupInvoiceParams`
- ffi: fix equality operator (`==`)
- js(nostr): fix `Keys` method calls in examples

### Removed

- Removed deprecated

## v0.29.4 - 2024/04/08

- pool: fix `InternalRelay::get_events_of_with_callback` timeout

## v0.29.3 - 2024/04/04

- pool: check filter limit in `InternalRelayPool::get_events_from`

## v0.29.2 - 2024/03/27

### Fixed

- pool: fix `get_events_of` issues

## v0.29.1 - 2024/03/26

### Fixed

- nostr: fix deserialization issues for events with non-standard `k` and `x` tags
- pool: fix spurious send_event timeout error (https://github.com/rust-nostr/nostr/pull/375)

<!-- Contributors -->
<!-- TODO: move contributors to a dedicated file -->
[Yuki Kishimoto]: <https://yukikishimoto.com> (nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet)
[DanConwayDev]: <https://github.com/DanConwayDev> (nostr:npub15qydau2hjma6ngxkl2cyar74wzyjshvl65za5k5rl69264ar2exs5cyejr)
[Daniel Cadenas]: <https://github.com/dcadenas> (nostr:npub138he9w0tumwpun4rnrmywlez06259938kz3nmjymvs8px7e9d0js8lrdr2)
[rustedmoon]: <https://github.com/rustedmoon> (nostr:npub1mheh5x5uhplms73kl73hwtg4gf57qxq89fvkwc2ykj8y966l05cqh9qtf9)
[benthecarman]: <https://github.com/benthecarman> (nostr:npub1u8lnhlw5usp3t9vmpz60ejpyt649z33hu82wc2hpv6m5xdqmuxhs46turz)
[Janek]: <https://github.com/xeruf> (nostr:npub1acxjpdrlk2vw320dxcy3prl87g5kh4c73wp0knullrmp7c4mc7nq88gj3j)
[Xiao Yu]: <https://github.com/kasugamirai> (nostr:npub1q0uulk2ga9dwkp8hsquzx38hc88uqggdntelgqrtkm29r3ass6fq8y9py9)
[RydalWater]: <https://github.com/RydalWater> (nostr:npub1zwnx29tj2lnem8wvjcx7avm8l4unswlz6zatk0vxzeu62uqagcash7fhrf)
[lnbc1QWFyb24]: <https://github.com/lnbc1QWFyb24> (nostr:npub1k95p0e36xx62mwjltdlsrrjunnqx464wlf969f9u3stvrq5dah4qgds3z7)
[reyamir]: <https://github.com/reyamir> (nostr:npub1zfss807aer0j26mwp2la0ume0jqde3823rmu97ra6sgyyg956e0s6xw445)
[w3irdrobot]: <https://github.com/w3irdrobot> (nostr:npub17q5n2z8naw0xl6vu9lvt560lg33pdpe29k0k09umlfxm3vc4tqrq466f2y)
[nanikamado]: <https://github.com/nanikamado> (?)
[rodant]: <https://github.com/rodant> (nostr:npub1w80jzxf36fhwgyfp622m6s7tcl3cy5z7xva4cy75q9kwm92zm8tsclzqjv)
[JeffG]: <https://github.com/erskingardner> (nostr:npub1zuuajd7u3sx8xu92yav9jwxpr839cs0kc3q6t56vd5u9q033xmhsk6c2uc)
[J. Azad EMERY]: <https://github.com/ethicnology> (?)
[v0l]: <https://github.com/v0l> (nostr:npub1v0lxxxxutpvrelsksy8cdhgfux9l6a42hsj2qzquu2zk7vc9qnkszrqj49)
[arkanoider]: <https://github.com/arkanoider> (nostr:npub1qqpn4ym6tc5ul6d2kjxnzx3sv9trekp53678ut9fe3wrxa6yvhjsnql2ng)
[1wErt3r]: <https://github.com/1wErt3r> (nostr:npub1xj5hzn62q2jg8xp9m3j6lw7r8z6g47plqyz2jmjr3g52y8tx4rls095s8g)
[dluvian]: <https://github.com/dluvian> (nostr:npub1useke4f9maul5nf67dj0m9sq6jcsmnjzzk4ycvldwl4qss35fvgqjdk5ks)
[Akiomi Kamakura]: https://github.com/akiomik (nostr:npub1f5uuywemqwlejj2d7he6zjw8jz9wr0r5z6q8lhttxj333ph24cjsymjmug)
[Darrell]: https://github.com/aki-mizu (nostr:npub1lu2qcwt23uq5pku99pxfe3uudpzdl4cfks24c2758cqqnfehujlqn6xlm6)
[Jens K.]: https://github.com/sectore (nostr:npub163jct20kzgjjr6z28u4vskax7d0gwq3zemrk6flgnw430vu55vtsdeqdc2)
[RandyMcMillan]: https://github.com/RandyMcMillan (nostr:npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy)
[Roland Bewick]: https://github.com/rolznz (nostr:npub1zk6u7mxlflguqteghn8q7xtu47hyerruv6379c36l8lxzzr4x90q0gl6ef)
[Francisco Calderón]: https://github.com/grunch (nostr:npub1qqqqqqqx2tj99mng5qgc07cgezv5jm95dj636x4qsq7svwkwmwnse3rfkq)
[cipres]: https://github.com/PancakesArchitect (nostr:npub1r3cnzta52fee26c83cnes8wvzkch3kud2kll67k402x04mttt26q0wfx0c)
[awiteb]: https://git.4rs.nl (nostr:nprofile1qqsqqqqqq9g9uljgjfcyd6dm4fegk8em2yfz0c3qp3tc6mntkrrhawgpzfmhxue69uhkummnw3ezudrjwvhxumq3dg0ly)
[magine]: https://github.com/ma233 (?)
[daywalker90]: https://github.com/daywalker90 (nostr:npub1kuemsj7xryp0uje36dr53scn9mxxh8ema90hw9snu46633n9n2hqp3drjt)
[Daniel D’Aquino]: https://github.com/danieldaquino (nostr:npub13v47pg9dxjq96an8jfev9znhm0k7ntwtlh9y335paj9kyjsjpznqzzl3l8)
[Thomas Profelt]: https://github.com/tompro (nostr:npub1rf0lc5dpyvpl6q3dfq0n0mtqc0maxa0kdehcj9nc5884fzufuzxqv67gj6)
