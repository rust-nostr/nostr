from nostr_protocol import Keys

def keys():
    keys = Keys.generate()
    public_key = keys.public_key()
    secret_key = keys.secret_key()

    print("Keys:")
    print(" Public keys:")
    print(f"     hex:    {public_key.to_hex()}")
    print(f"     bech32: {public_key.to_bech32()}")
    print(" Secret keys:")
    print(f"     hex:    {secret_key.to_hex()}")
    print(f"     bech32: {secret_key.to_bech32()}")
