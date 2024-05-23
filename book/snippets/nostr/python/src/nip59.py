from nostr_protocol import Keys, EventBuilder, Event, gift_wrap, UnwrappedGift, UnsignedEvent


def nip59():
    print("\nGift Wrapping (NIP-59):")
    # Sender Keys
    alice_keys = Keys.parse("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")

    # Receiver Keys
    bob_keys = Keys.parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")

    # Compose rumor
    rumor = EventBuilder.text_note("Test", []).to_unsigned_event(alice_keys.public_key())

    # Build gift wrap with sender keys
    gw: Event = gift_wrap(alice_keys, bob_keys.public_key(), rumor, None)
    print(f" Gift Wrap:\n{gw.as_json()}")

    # Extract rumor from gift wrap with receiver keys
    print("\n Unwrapped Gift:")
    unwrapped_gift = UnwrappedGift.from_gift_wrap(bob_keys, gw)
    sender = unwrapped_gift.sender()
    rumor: UnsignedEvent = unwrapped_gift.rumor()
    print(f"     Sender: {sender.to_bech32()}")
    print(f"     Rumor: {rumor.as_json()}")
