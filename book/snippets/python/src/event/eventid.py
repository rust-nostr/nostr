from nostr_sdk import EventId, Keys, Timestamp, Kind, EventBuilder


def event_id():
    keys = Keys.generate()

    print()
    print("Event ID:")

    # ANCHOR: build-event-id
    print("  Build Event ID:")
    event_id = EventId(keys.public_key(), Timestamp.now(), Kind(1), [], "content")
    print(f"     - {event_id}")
    # ANCHOR_END: build-event-id

    print()
    # ANCHOR: format-parse-hex
    # To Hex and then Parse
    print("  Event ID (hex):")
    event_id_hex = event_id.to_hex()
    print(f"     - Hex: {event_id_hex}")
    print(f"     - Parse: {EventId.parse(event_id_hex)}")
    print(f"     - From Hex: {EventId.from_hex(event_id_hex)}")
    # ANCHOR_END: format-parse-hex

    print()
    # ANCHOR: format-parse-bech32
    # To Bech32 and then Parse
    print("  Event ID (bech32):")
    event_id_bech32 = event_id.to_bech32()
    print(f"     - Bech32: {event_id_bech32}")
    print(f"     - Parse: {EventId.parse(event_id_bech32)}")
    print(f"     - From Bech32: {EventId.from_bech32(event_id_bech32)}")
    # ANCHOR_END: format-parse-bech32

    print()
    # ANCHOR: format-parse-nostr-uri
    # To Nostr URI and then Parse
    print("  Event ID (nostr uri):")
    event_id_nostr_uri = event_id.to_nostr_uri()
    print(f"     - Nostr URI: {event_id_nostr_uri}")
    print(f"     - Parse: {EventId.parse(event_id_nostr_uri)}")
    print(f"     - From Nostr URI: {EventId.from_nostr_uri(event_id_nostr_uri)}")
    # ANCHOR_END: format-parse-nostr-uri

    print()
    # ANCHOR: format-parse-bytes
    # As Bytes and then Parse
    print("  Event ID (bytes):")
    event_id_bytes = event_id.as_bytes()
    print(f"     - Bytes: {event_id_bytes}")
    print(f"     - From Bytes: {EventId.from_bytes(event_id_bytes)}")
    # ANCHOR_END: format-parse-bytes

    print()
    # ANCHOR: access-verify
    # Event ID from Event & Verfiy
    print("  Event ID from Event & Verify:")
    event = EventBuilder.text_note("This is a note", []).sign_with_keys(keys)
    print(f"     - Event ID: {event.id()}")
    print(f"     - Verify the ID & Signature: {event.verify()}")
    # ANCHOR_END: access-verify
