from nostr_protocol import Keys, Metadata, EventBuilder, PublicKey, verify_nip05, get_nip05_profile


async def nip05():
    # ANCHOR: set-metadata
    # Create metadata object with name and NIP05
    metadata = Metadata() \
        .set_name("TestName") \
        .set_nip05("TestName@rustNostr.com")
    # ANCHOR_END: set-metadata

    print()

    # ANCHOR: verify-nip05
    print("Verify NIP-05:")
    nip_05 = "yuki@yukikishimoto.com"
    public_key = PublicKey.parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
    proxy = None
    if await verify_nip05(public_key, nip_05, proxy):
        print(f"     '{nip_05}' verified, for {public_key.to_bech32()}")
    else:
        print(f"     Unable to verify NIP-05, for {public_key.to_bech32()}")
    # ANCHOR_END: verify-nip05

    print()

    # ANCHOR: nip05-profile
    print("Profile NIP-05:")
    nip_05 = "yuki@yukikishimoto.com"
    profile = await get_nip05_profile(nip_05)
    print(f"     {nip_05} Profile: {profile.to_bech32()}")
    # ANCHOR_END: nip05-profile
