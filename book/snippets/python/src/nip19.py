from nostr_sdk import Keys, EventBuilder, Nip19Profile, Nip19, Nip19Event, Coordinate, Kind
def nip19():
    keys = Keys.generate()

    print()
    print("Bare keys and ids (bech32):")
    # ANCHOR: nip19-npub
    print(f" Public key: {keys.public_key().to_bech32()}")
    # ANCHOR_END: nip19-npub

    # ANCHOR: nip19-nsec
    print(f" Secret key: {keys.secret_key().to_bech32()}")
    # ANCHOR_END: nip19-nsec

    # ANCHOR: nip19-note
    event = EventBuilder.text_note("Hello from rust-nostr Python bindings!").sign_with_keys(keys)
    print(f" Event     : {event.id().to_bech32()}")
    # ANCHOR_END: nip19-note

    print()
    print("Shareable identifiers with extra metadata (bech32):")
    # ANCHOR: nip19-nprofile-encode
    # Create NIP-19 profile including relays data
    relays = ["wss://relay.damus.io"]
    nprofile = Nip19Profile(keys.public_key(),relays)
    print(f" Profile (encoded): {nprofile.to_bech32()}")
    # ANCHOR_END: nip19-nprofile-encode

    # ANCHOR: nip19-nprofile-decode
    # Decode NIP-19 profile
    decode_nprofile = Nip19.from_bech32(nprofile.to_bech32())
    print(f" Profile (decoded): {decode_nprofile}")
    # ANCHOR_END: nip19-nprofile-decode

    print()
    # ANCHOR: nip19-nevent-encode
    # Create NIP-19 event including author and relays data
    nevent = Nip19Event(event.id(), keys.public_key(), kind=None, relays=relays)
    print(f" Event (encoded): {nevent.to_bech32()}")
    # ANCHOR_END: nip19-nevent-encode

    # ANCHOR: nip19-nevent-decode
    # Decode NIP-19 event
    decode_nevent = Nip19.from_bech32(nevent.to_bech32())
    print(f" Event (decoded): {decode_nevent}")
    # ANCHOR_END: nip19-nevent-decode

    print()
    # ANCHOR: nip19-naddr-encode
    # Create NIP-19 coordinate
    coord = Coordinate(Kind(0),keys.public_key())
    print(f" Coordinate (encoded): {coord.to_bech32()}")
    # ANCHOR_END: nip19-naddr-encode

    # ANCHOR: nip19-naddr-decode
    # Decode NIP-19 coordinate
    decode_coord = Nip19.from_bech32(coord.to_bech32())
    print(f" Coordinate (decoded): {decode_coord}")
    # ANCHOR_END: nip19-naddr-decode
