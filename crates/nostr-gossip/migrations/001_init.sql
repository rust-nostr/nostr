-- Database settings
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA user_version = 1; -- Schema version

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key BLOB NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS relays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    relay_url TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS lists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pkid INTEGER NOT NULL, -- Public Key ID
    kind INTEGER NOT NULL, -- Kind of the list (i.e., 10002 for NIP65, 10050 for NIP17)
    created_at INTEGER NOT NULL DEFAULT 0, -- Event list created at (`created_at` field of event)
    last_check INTEGER NOT NULL DEFAULT 0, -- The timestamp of the last check
    FOREIGN KEY(pkid) REFERENCES users(id) ON DELETE CASCADE ON UPDATE NO ACTION
);

CREATE UNIQUE INDEX IF NOT EXISTS pubkey_list_idx ON lists(pkid,kind);

CREATE TABLE IF NOT EXISTS relays_by_list (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pkid INTEGER NOT NULL, -- Public Key ID
    listid INTEGER NOT NULL, -- List ID
    relayid INTEGER NOT NULL, -- Relay ID
    metadata TEXT DEFAULT NULL, -- NIP65 metadata: read, write or NULL
    FOREIGN KEY(pkid) REFERENCES users(id) ON DELETE CASCADE ON UPDATE NO ACTION,
    FOREIGN KEY(listid) REFERENCES lists(id) ON DELETE CASCADE ON UPDATE NO ACTION,
    FOREIGN KEY(relayid) REFERENCES relays(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS pubkey_list_relay_idx ON relays_by_list(pkid,listid,relayid);

-- CREATE TABLE IF NOT EXISTS tracker (
--     id INTEGER PRIMARY KEY AUTOINCREMENT,
--     pkid INTEGER NOT NULL, -- Public Key ID
--     relayid INTEGER NOT NULL, -- Relay ID
--     last_event INTEGER NOT NULL DEFAULT 0, -- Timestamp of the last event seen for the public key on the relay
--     score INTEGER NOT NULL DEFAULT 5, -- Score
--     FOREIGN KEY(pkid) REFERENCES users(id) ON DELETE CASCADE ON UPDATE NO ACTION,
--     FOREIGN KEY(relayid) REFERENCES relays(id)
-- );
--
-- CREATE UNIQUE INDEX IF NOT EXISTS pubkey_relay_idx ON tracker(pkid,relayid);
