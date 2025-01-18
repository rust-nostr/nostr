from nostr_sdk import Keys, EventBuilder, PublicKey, Tag, TagStandard

keys = Keys.generate()

other_user_pk = PublicKey.parse("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")

tag = Tag.parse(["p", other_user_pk.to_hex()])
# OR
tag = Tag.from_standardized(TagStandard.PUBLIC_KEY_TAG(other_user_pk, None, None, False))
# OR
tag = Tag.public_key(other_user_pk)

event = EventBuilder.text_note("New note from Rust Nostr python bindings").tags([tag]).sign_with_keys(keys)
print(event.as_json())

print("\nTags:")
for tag in event.tags().to_vec():
    print(tag.as_vec())
