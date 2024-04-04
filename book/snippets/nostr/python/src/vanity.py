from nostr_protocol import Keys


def vanity():
    keys = Keys.vanity(["yuk0"], True, 8)

    print(f"Public keys: {keys.public_key().to_bech32()}")
    print(f"Secret keys: {keys.secret_key().to_bech32()}")
