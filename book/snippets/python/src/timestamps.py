from nostr_sdk import Timestamp, EventBuilder, Keys, Kind, gift_wrap, Tag


def timestamps():
    # Generate keys and Events
    alice_keys = Keys.generate()
    bob_keys = Keys.generate()

    print()
    print("Timestamps:")

    # ANCHOR: timestamp-now
    print("  Simple timestamp (now):")
    timestamp = Timestamp.now()
    print(f"     As str: {timestamp.to_human_datetime()}")
    print(f"     As int: {timestamp.as_secs()}")
    # ANCHOR_END: timestamp-now

    print()
    # ANCHOR: timestamp-parse
    print("  Parse timestamp (sec):")
    timestamp = Timestamp.from_secs(1718737479)
    print(f"     {timestamp.to_human_datetime()}")
    # ANCHOR_END: timestamp-parse

    print()
    # ANCHOR: timestamp-created
    print("  Created at timestamp:")
    event = EventBuilder(Kind(1), "This is some event text.").custom_created_at(timestamp).sign_with_keys(alice_keys)
    print(f"     Created at: {event.created_at().to_human_datetime()}")
    # ANCHOR_END: timestamp-created

    print()
    # ANCHOR: timestamp-tag
    print("  Timestamp Tag:")
    tag = Tag.expiration(timestamp)
    print(f"     Tag: {tag.as_standardized()}")
    # ANCHOR_END: timestamp-tag
