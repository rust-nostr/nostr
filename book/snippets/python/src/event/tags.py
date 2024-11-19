from typing import cast
from nostr_sdk import EventBuilder, Keys, Tag, Contact, Coordinate, Kind, RelayMetadata, TagKind


def tags():
    # Generate keys and events
    keys = Keys.generate()
    event = EventBuilder.contact_list([Contact(keys.public_key(), "", "")]).sign_with_keys(keys)

    print()
    print("Tags:")

    # ANCHOR: single-letter
    print("  Single Letter Tags:")
    # Event ID (hex)
    tag = Tag.event(event.id())
    print(f"     - Event ID (hex)     : {tag.as_vec()}")
    # Public Key (hex)
    tag = Tag.public_key(keys.public_key())
    print(f"     - Public Key (hex)   : {tag.as_vec()}")
    # Coordinate to event
    tag = Tag.coordinate(Coordinate(Kind(0), keys.public_key()))
    print(f"     - Coordinate to event: {tag.as_vec()}")
    # Identifier
    tag = Tag.identifier("This is an identifier value")
    print(f"     - Identifier         : {tag.as_vec()}")
    # Reference/Relay
    tag = Tag.relay_metadata("wss://relay.example.com",RelayMetadata.READ)
    print(f"     - Reference/Relays   : {tag.as_vec()}")
    # Hashtag
    tag = Tag.hashtag("#AskNostr")
    print(f"     - Hashtag            : {tag.as_vec()}")
    # ANCHOR_END: single-letter

    print()
    # ANCHOR: custom
    print("  Custom Tags:")
    tag = Tag.custom(cast(TagKind, TagKind.SUMMARY()), ["This is a summary"])
    print(f"     - Summary    : {tag.as_vec()}")
    tag = Tag.custom(cast(TagKind, TagKind.AMOUNT()), ["42"])
    print(f"     - Amount     : {tag.as_vec()}")
    tag = Tag.custom(cast(TagKind, TagKind.TITLE()), ["This is a title"])
    print(f"     - Title      : {tag.as_vec()}")
    tag = Tag.custom(cast(TagKind, TagKind.SUBJECT()), ["This is a subject"])
    print(f"     - Subject    : {tag.as_vec()}")
    tag = Tag.custom(cast(TagKind, TagKind.DESCRIPTION()), ["This is a description"])
    print(f"     - Description: {tag.as_vec()}")
    tag = Tag.custom(cast(TagKind, TagKind.URL()), ["https://example.com"])
    print(f"     - URL        : {tag.as_vec()}")
    # ANCHOR_END: custom

    print()
    # ANCHOR: parse
    print("  Parsing Tags:")
    tag = Tag.parse(["L","Label Namespace"])
    print(f"     - Label Namespace: {tag.as_vec()}")
    tag = Tag.parse(["l","Label Value"])
    print(f"     - Label Value    : {tag.as_vec()}")
    # ANCHOR_END: parse

    print()
    # ANCHOR: access
    print("  Working with Tags:")
    tag = Tag.public_key(keys.public_key())
    print(f"     - Kind     : {tag.kind()}")
    print(f"     - Letter   : {tag.single_letter_tag()}")
    print(f"     - Content  : {tag.content()}")
    print(f"     - As Std   : {tag.as_standardized()}")
    print(f"     - As Vector: {tag.as_vec()}")
    # ANCHOR_END: access

    print()
    # ANCHOR: logical
    print("  Logical Tests:")
    tag = Tag.custom(cast(TagKind, TagKind.SUMMARY()), ["This is a summary"])
    print(f"     - Tag1 (Title?)  : {tag.kind().is_title()}")
    print(f"     - Tag1 (Summary?): {tag.kind().is_summary()}")
    # ANCHOR_END: logical
