# Migrations

## Notes

SQLx creates a checksum of the migrations and compares it to the database.
This means that also comments are included in the checksum. If you change
comments, the hash will change and will break the migrations!

## SQL file format

- Use a tab for indentation
- Leave an empty line at the end of the file
- **DON'T use `--` comments** (schema comments are documented below)

## Schemas

### Public keys table

This table is used for storing the seen public keys.

Complete SQL schema:

```sql
CREATE TABLE public_keys(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key BLOB NOT NULL UNIQUE,
    CHECK (length(public_key) = 32)
);
```

Columns description:

- `id`: Public Key ID
- `public_key`: Public Key 32-byte array

### Lists table

This table is used for keeping track of user's lists (last checked at, kind, etc.)

Complete SQL schema:

```sql
CREATE TABLE lists(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key_id INTEGER NOT NULL,
    event_kind INTEGER NOT NULL,
    event_created_at INTEGER DEFAULT NULL,
    last_checked_at INTEGER DEFAULT NULL,
    UNIQUE(public_key_id, event_kind),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id) ON DELETE CASCADE
);
```

Columns description:

- `public_key_id`: Public Key ID
- `event_kind`: The event kind of the list (i.e., 10050, 10002)
- `event_created_at`: UNIX timestamp of when the event list has been created
- `last_checked_at`: UNIX timestamp of the last check

### Relays table

This table is used for storing the seen/received relays.

Complete SQL schema:

```sql
CREATE TABLE relays(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    CHECK (length(url) > 0)
);
```

Columns description:

- `id`: Relay ID
- `url`: Relay URL

### Relays-per-user table

This table is used for keeping track of user's relays (bitflags, received events, etc.)

Complete SQL schema:

```sql
CREATE TABLE relays_per_user(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key_id INTEGER NOT NULL,
    relay_id INTEGER NOT NULL,
    bitflags INTEGER NOT NULL DEFAULT 0,
    received_events INTEGER NOT NULL DEFAULT 0,
    last_received_event INTEGER NOT NULL DEFAULT 0,
    UNIQUE(public_key_id, relay_id),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id) ON DELETE CASCADE,
    FOREIGN KEY (relay_id) REFERENCES relays(id) ON DELETE CASCADE
);
```

Columns description:

- `public_key_id`: Public Key ID
- `relay_id`: Relay ID
- `bitflags`: flags of the relay (read, write, hint, etc.)
- `received_events`: number of received events from the relay for that user
- `last_received_event`: UNIX timestamp of the last received event from the relay for that user
