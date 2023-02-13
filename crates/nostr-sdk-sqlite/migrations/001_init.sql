-- Database settings
PRAGMA encoding = "UTF-8";
PRAGMA journal_mode=WAL;
PRAGMA main.synchronous=NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA application_id = 1654008667;
PRAGMA user_version = 1; -- Schema version

-- Relays Table
CREATE TABLE IF NOT EXISTS relays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    proxy TEXT DEFAULT NULL,
    enabled BOOLEAN DEFAULT TRUE
);

CREATE UNIQUE INDEX IF NOT EXISTS relays_url_index ON relays(url);

-- Events Table
CREATE TABLE IF NOT EXISTS events (
id BLOB PRIMARY KEY,
pubkey BLOB NOT NULL,
created_at INTEGER NOT NULL,
kind INTEGER NOT NULL,
content TEXT NOT NULL,
sig TEXT NOT NULL
);

-- Events Indexes
CREATE INDEX IF NOT EXISTS pubkey_index ON events(pubkey);
CREATE INDEX IF NOT EXISTS kind_index ON events(kind);
CREATE INDEX IF NOT EXISTS created_at_index ON events(created_at);
CREATE INDEX IF NOT EXISTS event_composite_index ON events(kind,created_at);
CREATE INDEX IF NOT EXISTS kind_pubkey_index ON events(kind,pubkey);
CREATE INDEX IF NOT EXISTS kind_created_at_index ON events(kind,created_at);
CREATE INDEX IF NOT EXISTS pubkey_created_at_index ON events(pubkey,created_at);
CREATE INDEX IF NOT EXISTS pubkey_kind_index ON events(pubkey,kind);

-- Tags Table
CREATE TABLE IF NOT EXISTS tags (
id INTEGER PRIMARY KEY,
event_id BLOB NOT NULL,
kind TEXT NOT NULL, -- the tag name ("p", "e", whatever)
value BLOB, -- tag contents
FOREIGN KEY(event_id) REFERENCES events(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Tags Indexes
CREATE INDEX IF NOT EXISTS tag_val_index ON tags(value);
CREATE UNIQUE INDEX IF NOT EXISTS tag_composite_index ON tags(event_id,kind,value);
CREATE UNIQUE INDEX IF NOT EXISTS tag_kind_eid_index ON tags(kind,event_id,value);
