-- The actual event data
CREATE TABLE IF NOT EXISTS events (
    id BLOB(32) PRIMARY KEY NOT NULL,
    pubkey BLOB(32) NOT NULL,
    created_at BIGINT NOT NULL,
    kind BIGINT NOT NULL,
    payload BLOB NOT NULL,
    deleted BOOLEAN NOT NULL
);

-- Direct indexes
CREATE INDEX event_pubkey ON events (pubkey);
CREATE INDEX event_date ON events (created_at);
CREATE INDEX event_kind ON events (kind);
CREATE INDEX event_deleted ON events (deleted);

-- The tag index, the primary will give us the index automatically
CREATE TABLE IF NOT EXISTS event_tags (
    tag VARCHAR(64) NOT NULL,
    tag_value VARCHAR(512) NOT NULL,
    event_id BLOB(32) NOT NULL
    REFERENCES events (id)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
    PRIMARY KEY (tag, tag_value, event_id)
);
