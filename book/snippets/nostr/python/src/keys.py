from nostr_protocol import Keys, SecretKey


# ANCHOR: generate
def generate():
    keys = Keys.generate()
    public_key = keys.public_key()
    secret_key = keys.secret_key()

    print("Keys:")
    print(" Public keys:")
    print(f"     hex:    {public_key.to_hex()}")
    print(f"     bech32: {public_key.to_bech32()}")
    print()
    print(" Secret keys:")
    print(f"     hex:    {secret_key.to_hex()}")
    print(f"     bech32: {secret_key.to_bech32()}")
# ANCHOR_END: generate

# ANCHOR: restore
def restore():
    keys = Keys.parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")

    secret_key = SecretKey.from_hex("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
    keys = Keys(secret_key)

    secret_key = SecretKey.from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
    keys = Keys(secret_key)
# ANCHOR_END: restore

print()
# ANCHOR: vanity
def vanity():
    keys = Keys.vanity(["yuk0"], True, 8)
    print(" Vanity:")
    print(f"     Public keys: {keys.public_key().to_bech32()}")
    print(f"     Secret keys: {keys.secret_key().to_bech32()}")
# ANCHOR_END: vanity
