from nostr import Keys

print("Mining keys...")
keys = Keys.vanity(["yuk0"], True, 8)

sk = keys.secret_key()
pk = keys.public_key()
print("Keys:")
print(" Public keys:")
print(f"     hex:    {pk.to_hex()}")
print(f"     bech32: {pk.to_bech32()}")
print(" Secret keys:")
print(f"     hex:    {sk.to_hex()}")
print(f"     bech32: {sk.to_bech32()}")
