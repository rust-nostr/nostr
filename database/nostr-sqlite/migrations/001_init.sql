CREATE TABLE events(
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    id BLOB NOT NULL UNIQUE,
    pubkey BLOB NOT NULL,
    created_at BIGINT NOT NULL,
    kind BIGINT NOT NULL,
    payload BLOB NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT 0
);

CREATE INDEX event_pubkey ON events(pubkey);
CREATE INDEX event_date ON events(created_at);
CREATE INDEX event_kind ON events(kind);
CREATE INDEX event_deleted ON events(deleted);

CREATE TABLE event_tags(
    tag TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    event_id BLOB NOT NULL
    REFERENCES events(id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (tag, tag_value, event_id)
);
