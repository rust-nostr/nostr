from nostr_protocol import Keys, Metadata, EventBuilder

def nip05():
    # Generate random keys
    keys = Keys.generate()

    # ANCHOR: create-event
    # Create metadata object with name and NIP05
    metadata_content = Metadata()\
        .set_name("TestName")\
        .set_nip05("TestName@rustNostr.com")

    # Build event and assign metadata
    builder = EventBuilder.metadata(metadata_content)

    # Signed event (Normal)
    print("\nCreating Metadata Event:")
    event = builder.to_event(keys)

    print(" Event Details:")
    print(f"     Kind      : {event.kind().as_u16()}")
    print(f"     Content   : {event.content()}")
    # ANCHOR_END: create-event

    # ANCHOR: create-metadata
    # Deserialize Metadata from event
    print("\nDeserializing Metadata Event:")
    metadata = Metadata().from_json(event.content())

    print(" Metadata Details:")
    print(f"     Name      : {metadata.get_name()}")
    print(f"     NIP05     : {metadata.get_nip05()}")
    # ANCHOR_END: create-metadata
