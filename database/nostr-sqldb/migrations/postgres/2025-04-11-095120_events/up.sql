-- The actual event data
CREATE TABLE events (
    id BYTEA PRIMARY KEY NOT NULL,
    pubkey BYTEA NOT NULL,
    created_at BIGINT NOT NULL,
    kind BIGINT NOT NULL,
    payload BYTEA NOT NULL,
    deleted BOOLEAN NOT NULL
);

-- Direct indexes
CREATE INDEX event_pubkey ON events (pubkey);
CREATE INDEX event_date ON events (created_at);
CREATE INDEX event_kind ON events (kind);
CREATE INDEX event_deleted ON events (deleted);

-- The tag index, the primary will give us the index automatically
CREATE TABLE event_tags (
    tag TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    event_id BYTEA NOT NULL
    REFERENCES events (id)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
    PRIMARY KEY (tag, tag_value, event_id)
);
