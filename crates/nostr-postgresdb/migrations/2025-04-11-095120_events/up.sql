-- Init the schema
CREATE SCHEMA IF NOT EXISTS nostr;

-- The actual event data
CREATE TABLE nostr.events (
    id VARCHAR(64) PRIMARY KEY,
    pubkey VARCHAR(64) NOT NULL,
    created_at BIGINT NOT NULL,
    kind BIGINT NOT NULL,
    payload BYTEA NOT NULL,
    signature VARCHAR(128) NOT NULL,
    deleted BOOLEAN NOT NULL
);

-- Direct indexes
CREATE INDEX event_pubkey ON nostr.events (pubkey);
CREATE INDEX event_date ON nostr.events (created_at);
CREATE INDEX event_kind ON nostr.events (kind);
CREATE INDEX event_deleted ON nostr.events (deleted);

-- The tag index, the primary will give us the index automatically
CREATE TABLE nostr.event_tags (
    tag TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    event_id VARCHAR(64) NOT NULL
    REFERENCES nostr.events (id)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
    PRIMARY KEY (tag, tag_value, event_id)
);
