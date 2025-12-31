CREATE TABLE vanished_public_keys (
    pubkey BLOB PRIMARY KEY NOT NULL CHECK(length(pubkey) = 32)
) WITHOUT ROWID;
