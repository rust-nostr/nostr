PRAGMA user_version = 1; -- Schema version

CREATE TABLE public_keys(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key BLOB NOT NULL UNIQUE,
    CHECK (length(public_key) = 32)
);

CREATE INDEX idx_public_keys_public_key ON public_keys(public_key);

CREATE TABLE lists(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key_id INTEGER NOT NULL,
    event_kind INTEGER NOT NULL,
    event_created_at INTEGER DEFAULT NULL,
    last_checked_at INTEGER DEFAULT NULL,
    UNIQUE(public_key_id, event_kind),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id) ON DELETE CASCADE
);

CREATE INDEX idx_lists_pub_kind ON lists(public_key_id, event_kind);

CREATE TABLE relays(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    CHECK (length(url) > 0)
);

CREATE INDEX idx_relays_url ON relays(url);

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

CREATE INDEX idx_rpu_pub_relay ON relays_per_user(public_key_id, relay_id);
CREATE INDEX idx_rpu_pub_flags_rank ON relays_per_user(public_key_id, bitflags, received_events DESC, last_received_event DESC);
CREATE INDEX idx_rpu_relay ON relays_per_user(relay_id);
