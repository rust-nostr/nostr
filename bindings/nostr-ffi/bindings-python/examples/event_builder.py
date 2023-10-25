from nostr_protocol import Keys, PublicKey, EventBuilder, Event, Tag

keys = Keys.generate()

# Build a text note
event = EventBuilder.new_text_note("New note from Rust Nostr python bindings", []).to_event(keys)
print(event.as_json())

# Build a DM
receiver_pk = PublicKey.from_bech32("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
event = EventBuilder.new_encrypted_direct_msg(keys, receiver_pk, "New note from Rust Nostr python bindings", None).to_event(keys)
print(event.as_json())

# Build a custom event
kind = 1234
content = "My custom content"
tags = []
builder = EventBuilder(kind, content, tags)

# Normal
event = builder.to_event(keys)
print(f"Event: {event.as_json()}")

# POW
event = builder.to_pow_event(keys, 20)
print(f"POW event: {event.as_json()}")

# Unsigned
event = builder.to_unsigned_event(keys.public_key())
print(f"Event: {event.as_json()}")