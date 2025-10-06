PRAGMA foreign_keys = ON;

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
    event_created_at BIGINT DEFAULT NULL,
    last_checked_at BIGINT DEFAULT NULL,
    UNIQUE(public_key_id, event_kind),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id) ON DELETE CASCADE
);

CREATE INDEX idx_lists_pub_kind ON lists(public_key_id, event_kind);

CREATE TABLE relays(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE
);

CREATE INDEX idx_relays_relay ON relays(url);

CREATE TABLE relays_per_user(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key_id INTEGER NOT NULL,
    relay_id INTEGER NOT NULL,
    bitflags INTEGER NOT NULL DEFAULT 0,
    received_events INTEGER NOT NULL DEFAULT 0,
    last_received_event BIGINT NOT NULL DEFAULT 0,
    UNIQUE(public_key_id, relay_id),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id) ON DELETE CASCADE,
    FOREIGN KEY (relay_id) REFERENCES relays(id) ON DELETE CASCADE
);

CREATE INDEX idx_rpu_pub_relay ON relays_per_user(public_key_id, relay_id);
CREATE INDEX idx_rpu_pub_rank ON relays_per_user(public_key_id, received_events DESC);
CREATE INDEX idx_rpu_relay ON relays_per_user(relay_id);
