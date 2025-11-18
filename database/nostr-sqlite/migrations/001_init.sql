CREATE TABLE events(
    id BLOB PRIMARY KEY NOT NULL,
    pubkey BLOB NOT NULL,
    created_at BIGINT NOT NULL,
    kind BIGINT NOT NULL,
    content TEXT NOT NULL,
    tags JSONB NOT NULL,
    sig BLOB NOT NULL
) WITHOUT ROWID;

CREATE INDEX idx_events_pubkey ON events(pubkey);
CREATE INDEX idx_events_created_at ON events(created_at DESC);
CREATE INDEX idx_events_kind ON events(kind);
CREATE INDEX idx_events_pubkey_created_at ON events(pubkey, created_at DESC);
CREATE INDEX idx_events_pubkey_kind_created_at ON events(pubkey, kind, created_at DESC);
CREATE INDEX idx_events_kind_created_at ON events(kind, created_at DESC);

CREATE TABLE event_tags(
    event_id BLOB NOT NULL,
    tag_name TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    PRIMARY KEY (tag_name, tag_value, event_id)
);

CREATE INDEX idx_tags_event_id ON event_tags(event_id);

CREATE TABLE deleted_ids (
    event_id BLOB PRIMARY KEY NOT NULL
) WITHOUT ROWID;

CREATE TABLE deleted_coordinates (
    pubkey BLOB NOT NULL,
    kind BIGINT NOT NULL,
    identifier TEXT NOT NULL,
    deleted_at BIGINT NOT NULL,
    PRIMARY KEY (pubkey, kind, identifier)
) WITHOUT ROWID;
