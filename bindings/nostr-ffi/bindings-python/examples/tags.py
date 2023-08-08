from nostr_protocol import Keys, EventBuilder, PublicKey, Tag, TagEnum

keys = Keys.generate()

other_user_pk = PublicKey.from_bech32("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")

tag = Tag.parse(["p", other_user_pk.to_hex()])
# OR
tag = Tag.from_enum(TagEnum.PUB_KEY(other_user_pk.to_hex(), None))

event = EventBuilder.new_text_note("New note from Rust Nostr python bindings", [tag]).to_event(keys)
print(event.as_json())

print("\nTags:")
for tag in event.tags():
    print(tag.as_vec())
    # OR handle it as enum
    # TODO