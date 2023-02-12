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
