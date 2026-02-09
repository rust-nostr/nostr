PRAGMA user_version = 2; -- Schema version

CREATE TABLE vanished_public_keys (
    pubkey BLOB PRIMARY KEY NOT NULL CHECK(length(pubkey) = 32) -- The public key that has requested to vanish via NIP-62
) WITHOUT ROWID;
