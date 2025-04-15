-- Initial database schema for nostr-mls-sqlite-storage

-- Groups table
CREATE TABLE IF NOT EXISTS groups (
    mls_group_id BLOB PRIMARY KEY,
    nostr_group_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    admin_pubkeys JSONB NOT NULL,
    last_message_id TEXT,
    last_message_at INTEGER,
    group_type TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    state TEXT NOT NULL
);

-- Create unique index on nostr_group_id
CREATE UNIQUE INDEX IF NOT EXISTS idx_groups_nostr_group_id ON groups(nostr_group_id);

-- Group Relays table
CREATE TABLE IF NOT EXISTS group_relays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mls_group_id BLOB NOT NULL,
    relay_url TEXT NOT NULL,
    FOREIGN KEY (mls_group_id) REFERENCES groups(mls_group_id) ON DELETE CASCADE,
    UNIQUE(mls_group_id, relay_url)
);

-- Create index on mls_group_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_group_relays_mls_group_id ON group_relays(mls_group_id);

-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,  -- Event ID
    pubkey TEXT NOT NULL,
    kind INTEGER NOT NULL,
    mls_group_id BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    content TEXT NOT NULL,
    tags JSONB NOT NULL,
    event JSONB NOT NULL,
    wrapper_event_id TEXT NOT NULL,
    tokens JSONB NOT NULL,
    FOREIGN KEY (mls_group_id) REFERENCES groups(mls_group_id) ON DELETE CASCADE
);

-- Create indexes on messages table
CREATE INDEX IF NOT EXISTS idx_messages_mls_group_id ON messages(mls_group_id);
CREATE INDEX IF NOT EXISTS idx_messages_wrapper_event_id ON messages(wrapper_event_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_messages_pubkey ON messages(pubkey);
CREATE INDEX IF NOT EXISTS idx_messages_kind ON messages(kind);

-- Processed Messages table
CREATE TABLE IF NOT EXISTS processed_messages (
    wrapper_event_id TEXT PRIMARY KEY,
    message_event_id TEXT,
    processed_at INTEGER NOT NULL,
    state TEXT NOT NULL,
    failure_reason TEXT NOT NULL
);

-- Create index on message_event_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_processed_messages_message_event_id ON processed_messages(message_event_id);
CREATE INDEX IF NOT EXISTS idx_processed_messages_state ON processed_messages(state);
CREATE INDEX IF NOT EXISTS idx_processed_messages_processed_at ON processed_messages(processed_at);
CREATE INDEX IF NOT EXISTS idx_processed_messages_wrapper_event_id ON processed_messages(wrapper_event_id);

-- Welcome messages table
CREATE TABLE IF NOT EXISTS welcomes (
    id TEXT PRIMARY KEY,  -- Event ID
    event JSONB NOT NULL,
    mls_group_id BLOB NOT NULL,
    nostr_group_id TEXT NOT NULL,
    group_name TEXT NOT NULL,
    group_description TEXT NOT NULL,
    group_admin_pubkeys JSONB NOT NULL,
    group_relays JSONB NOT NULL,
    welcomer TEXT NOT NULL,  -- pubkey
    member_count INTEGER NOT NULL,
    state TEXT NOT NULL,
    wrapper_event_id TEXT NOT NULL,
    FOREIGN KEY (mls_group_id) REFERENCES groups(mls_group_id) ON DELETE CASCADE
);

-- Create indexes on welcomes table
CREATE INDEX IF NOT EXISTS idx_welcomes_mls_group_id ON welcomes(mls_group_id);
CREATE INDEX IF NOT EXISTS idx_welcomes_wrapper_event_id ON welcomes(wrapper_event_id);
CREATE INDEX IF NOT EXISTS idx_welcomes_state ON welcomes(state);
CREATE INDEX IF NOT EXISTS idx_welcomes_nostr_group_id ON welcomes(nostr_group_id);

-- Processed Welcome messages table
CREATE TABLE IF NOT EXISTS processed_welcomes (
    wrapper_event_id TEXT PRIMARY KEY,
    welcome_event_id TEXT,
    processed_at INTEGER NOT NULL,
    state TEXT NOT NULL,
    failure_reason TEXT NOT NULL
);

-- Create index on welcome_event_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_processed_welcomes_welcome_event_id ON processed_welcomes(welcome_event_id);
CREATE INDEX IF NOT EXISTS idx_processed_welcomes_state ON processed_welcomes(state);
CREATE INDEX IF NOT EXISTS idx_processed_welcomes_processed_at ON processed_welcomes(processed_at);
CREATE INDEX IF NOT EXISTS idx_processed_welcomes_wrapper_event_id ON processed_welcomes(wrapper_event_id);
