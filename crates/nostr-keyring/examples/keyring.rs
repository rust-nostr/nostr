// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_keyring::prelude::*;

fn main() -> Result<()> {
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;

    let keyring = NostrKeyring::new("rust-nostr-test");

    keyring.set("test", &keys)?;

    let found_keys = keyring.get("test")?;

    assert_eq!(keys, found_keys);

    Ok(())
}
