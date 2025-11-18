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

### Events table

This table is used for storing the events.

Complete SQL schema:

```sql
CREATE TABLE events(
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 32),
    pubkey BLOB NOT NULL CHECK(length(pubkey) = 32),
    created_at INTEGER NOT NULL CHECK(created_at > 0),
    kind INTEGER NOT NULL CHECK(kind >= 0 AND kind <= 65535),
    content TEXT NOT NULL,
    tags JSONB NOT NULL,
    sig BLOB NOT NULL CHECK(length(sig) = 64)
) WITHOUT ROWID;

CREATE INDEX idx_events_pubkey ON events(pubkey);
CREATE INDEX idx_events_created_at ON events(created_at DESC);
CREATE INDEX idx_events_kind ON events(kind);
CREATE INDEX idx_events_pubkey_created_at ON events(pubkey, created_at DESC);
CREATE INDEX idx_events_pubkey_kind_created_at ON events(pubkey, kind, created_at DESC);
CREATE INDEX idx_events_kind_created_at ON events(kind, created_at DESC);
```

### Event tags table

This table is used for keeping track of user's single-letter tags.

Complete SQL schema:

```sql
CREATE TABLE event_tags(
    event_id BLOB NOT NULL CHECK(length(event_id) = 32),
    tag_name TEXT NOT NULL CHECK(length(tag_name) >= 1),                    -- Single-letter tag
    tag_value TEXT NOT NULL,                                                -- Tag content (value at index 1 of the array)
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    PRIMARY KEY (tag_name, tag_value, event_id)
);

CREATE INDEX idx_tags_event_id ON event_tags(event_id);
```

### Deleted events tables

For keeping track of the deleted events, we use two tables: `deleted_ids` and `deleted_coordinates`.

Complete SQL schema:

```sql
CREATE TABLE deleted_ids (
    event_id BLOB PRIMARY KEY NOT NULL CHECK(length(event_id) = 32)
) WITHOUT ROWID;

CREATE TABLE deleted_coordinates (
    pubkey BLOB NOT NULL CHECK(length(pubkey) = 32),
    kind INTEGER NOT NULL CHECK(kind >= 0 AND kind <= 65535),               -- Event kind (replaceable or addressable)
    identifier TEXT NOT NULL,                                               -- d tag, if it's an addressable event
    deleted_at INTEGER NOT NULL CHECK(deleted_at > 0),
    PRIMARY KEY (pubkey, kind, identifier)
) WITHOUT ROWID;

```
