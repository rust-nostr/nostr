# Nostr SQLite database backend

SQLite storage backend for nostr apps.

## Crate Feature Flags

The following crate feature flags are available:

| Feature   | Default | Description         |
|-----------|:-------:|---------------------|
| `bundled` |   Yes   | Uses bundled SQLite |

## Supported NIPs

| Supported | NIP                                                                                   |
|:---------:|---------------------------------------------------------------------------------------|
|     ❌     | [40 - Expiration Timestamp](https://github.com/nostr-protocol/nips/blob/master/40.md) |
|     ✅     | [50 - Search Capability](https://github.com/nostr-protocol/nips/blob/master/50.md)    |
|    ✅*     | [62 - Request to Vanish](https://github.com/nostr-protocol/nips/blob/master/62.md)    |

*: `ALL_RELAYS` only

## Example for wasm32-unknown-unknown

Add `sqlite-wasm-rs` and `sqlite-wasm-vfs` to your `Cargo.toml`.

```rust,no_run,ignore
use nostr_sqlite::prelude::*;
use sqlite_wasm_rs::WasmOsCallback;
use sqlite_wasm_vfs::relaxed_idb::{RelaxedIdbCfgBuilder, install as install_relaxed_idb_vfs};
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfgBuilder, install as install_opfs_sahpool_vfs};

const SQLITE_DB_PATH: &str = "my-db-name.sqlite3";
const SQLITE_OPFS_VFS: &str = "opfs-sahpool";
const SQLITE_IDB_VFS: &str = "relaxed-idb";

async fn register_opfs_vfs() -> Result<(), Box<dyn std::error::Error>> {
    let options = OpfsSAHPoolCfgBuilder::new()
        .vfs_name(SQLITE_OPFS_VFS)
        .directory(".mydir-opfs")
        .build();
    install_opfs_sahpool_vfs::<WasmOsCallback>(&options, false).await?;
    Ok(())
}

async fn register_idb_vfs() -> Result<(), Box<dyn std::error::Error>> {
    let options = RelaxedIdbCfgBuilder::new().vfs_name(SQLITE_IDB_VFS).build();
    install_relaxed_idb_vfs::<WasmOsCallback>(&options, false).await?;
    Ok(())
}

async fn open_wasm_db() -> NostrSqlite {
    if register_opfs_vfs().await.is_ok() {
        return NostrSqlite::open_with_vfs(SQLITE_DB_PATH, SQLITE_OPFS_VFS)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "failed to open sqlite db at `{SQLITE_DB_PATH}` with vfs `{}`: {err}",
                    SQLITE_OPFS_VFS
                )
            });
    }

    if register_idb_vfs().await.is_ok() {
        return NostrSqlite::open_with_vfs(SQLITE_DB_PATH, SQLITE_IDB_VFS)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "failed to open sqlite db at `{SQLITE_DB_PATH}` with vfs `{}`: {err}",
                    SQLITE_IDB_VFS
                )
            });
    }

    eprintln!(
        "SQLite is running in memory mode. Data is not persistent and will be lost on refresh."
    );

    NostrSqlite::in_memory()
        .await
        .unwrap_or_else(|err| panic!("failed to open in-memory sqlite in wasm: {err}"))
}
```

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
