# Nostr SQL database backend

SQL storage backend for nostr apps working with Postgres, SQLite and MySQL.

## Crate Feature Flags

The following crate feature flags are available:

| Feature     | Default | Description                   |
|-------------|:-------:|-------------------------------|
| `postgres`  |   Yes   | Enable support for PostgreSQL |
| `mysql`     |   No    | Enable support for MySQL      |
| `sqlite`    |   No    | Enable support for SQLite     |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
