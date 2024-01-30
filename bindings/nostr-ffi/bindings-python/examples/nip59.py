from nostr_protocol import Keys, PublicKey, EventBuilder, extract_rumor_from_gift_wrap

alice_keys = Keys.from_sk_str("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")
bob_keys = Keys.from_sk_str("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")

rumor = EventBuilder.new_text_note("Test", []).to_unsigned_event(alice_keys.public_key())
giftwrap = EventBuilder.gift_wrap(alice_keys, bob_keys.public_key(), rumor).to_event(alice_keys)
print(f"Gift Wrap: {giftwrap.as_json()}")

rumor = extract_rumor_from_gift_wrap(bob_keys, giftwrap)
print(f"Rumor: {rumor.as_json()}")