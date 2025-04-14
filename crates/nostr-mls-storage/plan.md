# Storage traits plan / structure

## OpenMLS

- `StorageProvider` - MLS storage traits.
- `MemoryStorage` - implementation of the StorageProvider traits for an in-memory data store https://github.com/openmls/openmls/tree/main/memory_storage
- `SqliteStorage` - implementation of the StorageProvider traits for a sqlite backed data store https://github.com/openmls/openmls/tree/main/sqlite_storage

## NostrMls

- Entrypoint for apps wanting to implement mls messaging in nostr clients
- Instantiates with a type that implements the `NostrMlsStorageProvider` traits

## NostrMlsStorage

- `NostrMlsStorageProvider` - trait defining my new methods that add extra functionality (methods to access groups, invites, messages, etc)

<br />

> ⚠️ **Support for Multiple Accounts**
>
> This setup doesn't have any concept of accounts ownership. This is left up to the implementing application (e.g. Whitenoise).

<br />

## Implementations Of `NostrMlsStorageProvider`

Each of these implementations also instantiates an implemenation of the `StorageProvider` trait and stores it in an `openmls_storage` field that is a generic of a type that implements StorageProvider

### NostrMlsMemoryStorage

- Implements in-memory storage using OpenMls `MemoryStorage` and an in-memory implementation of the `NostrMlsStorageProvider` traits.

### NostrMlsSqliteStorage

- Implements in-memory storage using OpenMls `SqliteStorage` and a sqlite implementation of the `NostrMlsStorageProvider` traits.
- Instantiates based on a file path so applications can create individual databases for each user account.

## Example Pseudocode

```rust
trait NostrMlsStorageProvider {
    // my additional methods for accessing groups, messages, invites, etc.
}

// Wrapper struct to delegate calls and add new functionality.
struct NostrMlsStorage<T>: NostrMlsStorageProvider
where T: StorageProvider {
    openmls_storage: T,
}

impl NostrMlsStorage<T> {
    fn new(mls_storage_implementation: T) -> Self {
        NostrMlsStorage {
            openmls_storage: mls_storage_implementation
        }
    }
}

// implementation of my custom trait methods
impl NostrMlsStorageProvider for NostrMlsStorage<T> {
    // my additional methods for accessing groups, messages, invites, etc.
}
```

## Usage:

```rust
let mls_datastore = NostrMlsStorage::new(memory_storage_implementation);
let nostr_mls = NostrMls::new(mls_datastore)

// Call NostrMlsStorageProvider methods from client application by calling nostr_mls.storage.method()

// Calls to underlying OpenMLS StorageProvider methods will happen in delegated methods in NostrMlsStorage so from the client application you'd call something like nostr_mls.create_group() and then that method would interact with the openmls & nostr-mls-storage methods directly.
```

# Database schema

## Stays in whitenoise

- Accounts
- Account Relays
- Media Files

_Remove any FK references to stuff that moves to nostr-mls_

## Moves to nostr-mls

- Groups
- Invites
- Messages
- Processed Invites - these track which giftwrapped events we've already processed so we don't re-process them
- Processed Messages - these track which giftwrapped events we've already processed so we don't re-process them
- Group Relays

_Remove `account_pubkey` fields and indexes_

# Crate Structure

- `nostr-mls`: The main crate that users will include when they want to use mls in their nostr apps.
- `nostr-mls-storage`: crate holding the NostrMlsStorageProvider trait, shared code for all implementations.
- `nostr-mls-memory-storage`: crate containing the in-memory implementation of the `NostrMlsStorageProvider` trait.
- `nostr-mls-sqlite-storage`: crate containing the sqlite implementation of the `NostrMlsStorageProvider` trait.
