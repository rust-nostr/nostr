CREATE TABLE public_keys(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key BLOB UNIQUE,
    last_nip17_update BIGINT,
    last_nip65_update BIGINT
);

CREATE TABLE relays(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key_id INTEGER,
    relay BLOB,
    bitflags BLOB,
    UNIQUE(public_key_id, relay),
    FOREIGN KEY(public_key_id) REFERENCES public_keys(id) ON DELETE CASCADE
);
