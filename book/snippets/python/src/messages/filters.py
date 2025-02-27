from nostr_sdk import Filter, FilterRecord, Keys, Kind, EventBuilder, Timestamp, Tag
import time, datetime


def filters():
    # Generate keys and Events
    keys = Keys.generate()
    keys2 = Keys.generate()
    event = EventBuilder.text_note("Hello World!").sign_with_keys(keys)
    event2 = EventBuilder(Kind(1), "Goodbye World!").tags([Tag.identifier("Identification D Tag")]).sign_with_keys(keys2)

    print()
    print("Creating Filters:")

    # ANCHOR: create-filter-id
    # Filter for specific ID
    print("  Filter for specific Event ID:")
    f = Filter().id(event.id())
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-id

    print()
    # ANCHOR: create-filter-author
    # Filter for specific Author
    print("  Filter for specific Author:")
    f = Filter().author(keys.public_key())
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-author

    print()
    # ANCHOR: create-filter-kind-pk
    # Filter by PK and Kinds
    print("  Filter with PK and Kinds:")
    f = Filter()\
        .pubkey(keys.public_key())\
        .kind(Kind(1))
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-kind-pk

    print()
    # ANCHOR: create-filter-search
    # Filter for specific string
    print("  Filter for specific search string:")
    f = Filter().search("Ask Nostr Anything")
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-search

    print()
    # ANCHOR: create-filter-timeframe
    print("  Filter for events from specific public key within given timeframe:")
    # Create timestamps
    date = datetime.datetime(2009, 1, 3, 0, 0)
    timestamp = int(time.mktime(date.timetuple()))
    since_ts = Timestamp.from_secs(timestamp)
    until_ts = Timestamp.now()

    # Filter with timeframe
    f = Filter()\
        .pubkey(keys.public_key())\
        .since(since_ts)\
        .until(until_ts)
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-timeframe

    print()
    # ANCHOR: create-filter-limit
    # Filter for specific PK with limit
    print("  Filter for specific Author, limited to 10 Events:")
    f = Filter()\
        .author(keys.public_key())\
        .limit(10)
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-limit

    print()
    # ANCHOR: create-filter-hashtag
    # Filter for Hashtags
    print("  Filter for a list of Hashtags:")
    f = Filter().hashtags(["#Bitcoin", "#AskNostr", "#Meme"])
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-hashtag

    print()
    # ANCHOR: create-filter-reference
    # Filter for Reference
    print("  Filter for a Reference:")
    f = Filter().reference("This is my NIP-12 Reference")
    print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-reference

    print()
    # ANCHOR: create-filter-identifier
    # Filter for Identifier
    print("  Filter for a Identifier:")
    identifier = event2.tags().identifier()
    if identifier is not None:
        f = Filter().identifier(identifier)
        print(f"     {f.as_json()}")
    # ANCHOR_END: create-filter-identifier

    print()
    print("Modifying Filters:")
    # ANCHOR: modify-filter
    # Modifying Filters (adding/removing)
    f = Filter()\
        .pubkeys([keys.public_key(), keys2.public_key()])\
        .ids([event.id(), event2.id()])\
        .kinds([Kind(0), Kind(1)])\
        .author(keys.public_key())

    # Add an additional Kind to existing filter
    f = f.kinds([Kind(4)])

    # Print Results
    print("  Before:")
    print(f"     {f.as_json()}")
    print()

    # Remove PKs, Kinds and IDs from filter
    f = f.remove_pubkeys([keys2.public_key()])
    print(" After (remove pubkeys):")
    print(f"     {f.as_json()}")

    f = f.remove_kinds([Kind(0), Kind(4)])
    print("  After (remove kinds):")
    print(f"     {f.as_json()}")

    f = f.remove_ids([event2.id()])
    print("  After (remove IDs):")
    print(f"     {f.as_json()}")
    # ANCHOR_END: modify-filter

    print()
    print("Other Filter Operations:")
    # ANCHOR: other-parse
    # Parse filter
    print("  Parse Filter from Json:")
    f_json = f.as_json()
    f = Filter().from_json(f_json)
    print(f"     {f.as_record()}")
    # ANCHOR_END: other-parse

    print()
    # ANCHOR: other-record
    print("  Construct Filter Record and extract author:")
    # Filter Record
    fr = FilterRecord(ids=[event.id()],authors=[keys.public_key()], kinds=[Kind(0)], search="", since=None, until=None, limit=1, generic_tags=[])
    f = Filter().from_record(fr)
    print(f"     {f.as_json()}")
    # ANCHOR_END: other-record

    print()
    # ANCHOR: other-match
    print("  Logical tests:")
    f = Filter().author(keys.public_key()).kind(Kind(1))
    print(f"     Event match for filter: {f.match_event(event)}")
    print(f"     Event2 match for filter: {f.match_event(event2)}")
    # ANCHOR_END: other-match

if __name__ == '__main__':
   filters()