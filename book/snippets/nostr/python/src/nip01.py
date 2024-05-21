from nostr_protocol import Keys, Metadata, EventBuilder
def nip01():
    # Generate random keys
    keys = Keys.generate()

    # ANCHOR: create-event
    # Create metadata object with desired content
    metadata_content = Metadata()\
        .set_name("TestName")\
        .set_display_name("PyTestur")\
        .set_about("This is a Test Account for Rust Nostr Python Bindings")\
        .set_website("https://rust-nostr.org/")\
        .set_picture("https://avatars.githubusercontent.com/u/123304603?s=200&v=4")\
        .set_banner("https://nostr-resources.com/assets/images/cover.png")\
        .set_nip05("TestName@rustNostr.com")

    # Build metadata event and assign content
    builder = EventBuilder.metadata(metadata_content)

    # Signed event and print details
    print("\nCreating Metadata Event:")
    event = builder.to_event(keys)

    print(" Event Details:")
    print(f"     Author    : {event.author().to_bech32()}")
    print(f"     Kind      : {event.kind().as_u16()}")
    print(f"     Content   : {event.content()}")
    print(f"     Datetime  : {event.created_at().to_human_datetime()}")
    print(f"     Signature : {event.signature()}")
    print(f"     Verify    : {event.verify()}")
    print(f"     JSON      : {event.as_json()}")
    # ANCHOR_END: create-event

    # ANCHOR: create-metadata
    # Deserialize Metadata from event
    print("\nDeserializing Metadata Event:")
    metadata = Metadata().from_json(event.content())
    
    print(" Metadata Details:")
    print(f"     Name      : {metadata.get_name()}")
    print(f"     Display   : {metadata.get_display_name()}")
    print(f"     About     : {metadata.get_about()}")
    print(f"     Website   : {metadata.get_website()}")
    print(f"     Picture   : {metadata.get_picture()}")
    print(f"     Banner    : {metadata.get_banner()}")
    print(f"     NIP05     : {metadata.get_nip05()}")
    # ANCHOR_END: create-metadata

if __name__ == "__main__":
    nip01()

