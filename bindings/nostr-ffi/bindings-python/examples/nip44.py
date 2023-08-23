from nostr_protocol import Keys, PublicKey, Nip44Version, nip44_decrypt, nip44_encrypt

alice_keys = Keys.from_sk_str("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")
bob_keys = Keys.from_sk_str("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")

payload = nip44_encrypt(alice_keys.secret_key(), bob_keys.public_key(), "hello", Nip44Version.X_CHA_CHA20)
print(f"Payload: {payload}")

plaintext = nip44_decrypt(bob_keys.secret_key(), alice_keys.public_key(), payload)
print(f"Plain text: {plaintext}")