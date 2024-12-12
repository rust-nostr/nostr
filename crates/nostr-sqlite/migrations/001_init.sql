-- Database settings
PRAGMA encoding = "UTF-8";
PRAGMA journal_mode=WAL;
PRAGMA main.synchronous=NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA application_id = 1654008667;
PRAGMA user_version = 1; -- Schema version

CREATE TABLE IF NOT EXISTS events (
    event_id BLOB PRIMARY KEY NOT NULL,
    event BLOB NOT NULL
);
