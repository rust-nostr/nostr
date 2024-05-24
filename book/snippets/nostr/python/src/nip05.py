from nostr_protocol import Keys, Metadata, EventBuilder, PublicKey, verify_nip05, get_nip05_profile

def nip05():
    # Generate random keys
    keys = Keys.generate()

    # ANCHOR: set-metadata
    # Create metadata object with name and NIP05
    metadata_content = Metadata()\
        .set_name("TestName")\
        .set_nip05("TestName@rustNostr.com")
    # ANCHOR_END: set-metadata

    # Build event and assign metadata
    builder = EventBuilder.metadata(metadata_content)

    # Signed event (Normal)
    print("\nCreating NIP-05 Metadata Event:")
    event = builder.to_event(keys)

    print(" Event Details:")
    print(f"     Kind      : {event.kind().as_u16()}")
    print(f"     Content   : {event.content()}")
    
    # ANCHOR: verify-nip05
    print("\nVerify NIP-05:")
    nip_05 = "yuki@yukikishimoto.com"
    public_key = PublicKey.parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
    proxy = None
    if verify_nip05(public_key, nip_05, proxy):
        print(f"     {nip_05} Verified, for {public_key.to_bech32()}")
    else:
        print(f"     Unable to Verifiy NIP-05, for {public_key.to_bech32()}")
    # ANCHOR_END: verify-nip05

    #ANCHOR: nip05-profile
    print("\nProfile NIP-05:")
    nip_05 = "Rydal@gitlurker.info"
    profile = get_nip05_profile(nip_05)
    print(f"     {nip_05} Profile: {profile.to_bech32()}")
    public_key = PublicKey.parse("npub1zwnx29tj2lnem8wvjcx7avm8l4unswlz6zatk0vxzeu62uqagcash7fhrf")
    #ANCHOR_END: nip05-profile