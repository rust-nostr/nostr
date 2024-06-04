from nostr_protocol import EventBuilder, Tag, Kind, Keys, RelayMetadata

def nip65():
    # Get Keys
    keys = Keys.generate()

    print()
    print("Relay Metadata:")
    # ANCHOR: relay-metadata-simple
    # Create relay dictionary
    relays_dict = {
        "wss://relay.damus.io": RelayMetadata.READ,
        "wss://relay.primal.net": RelayMetadata.WRITE,
        "wss://relay.nostr.band": None
    }
    
    # Build/sign event
    builder = EventBuilder.relay_list(relays_dict)
    event = builder.to_event(keys)

    # Print event as json
    print(f" Event: {event.as_json()}")
    # ANCHOR_END: relay-metadata-simple

    print()

    # ANCHOR: relay-metadata-custom
    # Create relay metadata tags
    tag1 = Tag.relay_metadata("wss://relay.damus.io", RelayMetadata.READ)
    tag2 = Tag.relay_metadata("wss://relay.primal.net", RelayMetadata.WRITE)
    tag3 = Tag.relay_metadata("wss://relay.nostr.band", None)

    # Build/sign event
    kind = Kind(10002)
    content = ""
    tags = [tag1,tag2,tag3]
    builder = EventBuilder(kind,content,tags)
    event = builder.to_event(keys)

    # Print event as json
    print(f" Event: {event.as_json()}")
    # ANCHOR_END: relay-metadata-custom