PRAGMA user_version = 1; -- Schema version

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

CREATE TABLE event_tags(
    event_id BLOB NOT NULL CHECK(length(event_id) = 32),
    tag_name TEXT NOT NULL CHECK(length(tag_name) >= 1), -- Single-letter tag
    tag_value TEXT NOT NULL,                             -- Tag content (value at index 1 of the array)
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    PRIMARY KEY (tag_name, tag_value, event_id)
);

CREATE INDEX idx_tags_event_id ON event_tags(event_id);

CREATE TABLE deleted_ids (
    event_id BLOB PRIMARY KEY NOT NULL CHECK(length(event_id) = 32)
) WITHOUT ROWID;

CREATE TABLE deleted_coordinates (
    pubkey BLOB NOT NULL CHECK(length(pubkey) = 32),
    kind INTEGER NOT NULL CHECK(kind >= 0 AND kind <= 65535), -- Event kind (replaceable or addressable)
    identifier TEXT NOT NULL,                                 -- d tag, if it's an addressable event
    deleted_at INTEGER NOT NULL CHECK(deleted_at > 0),
    PRIMARY KEY (pubkey, kind, identifier)
) WITHOUT ROWID;
