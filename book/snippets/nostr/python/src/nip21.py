from nostr_protocol import Keys, PublicKey, EventBuilder, EventId, Nip21, Nip19Profile, Nip19Event, Kind, Coordinate


def nip21():
    print()
    print("Nostr URIs:")
    # ANCHOR: npub
    keys = Keys.generate()

    # URI npub
    pk_uri = keys.public_key().to_nostr_uri()
    print(f" Public key (URI):    {pk_uri}")

    # bech32 npub
    pk_parse = Nip21.parse(pk_uri)
    if pk_parse.as_enum().is_pubkey():
        pk_bech32 = PublicKey.from_nostr_uri(pk_uri).to_bech32()
        print(f" Public key (bech32): {pk_bech32}")
    # ANCHOR_END: npub

    print()

    # ANCHOR: note
    event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_event(keys)

    # URI note
    note_uri = event.id().to_nostr_uri()
    print(f" Event (URI):    {note_uri}")

    # bech32 note
    note_pasre = Nip21.parse(note_uri)
    if note_pasre.as_enum().is_note():
        event_bech32 = EventId.from_nostr_uri(note_uri).to_bech32()
        print(f" Event (bech32): {event_bech32}")
    # ANCHOR_END: note

    print()

    # ANCHOR: nprofile
    relays = ["wss://relay.damus.io"]
    nprofile = Nip19Profile(keys.public_key(), relays)

    # URI nprofile
    nprofile_uri = nprofile.to_nostr_uri()
    print(f" Profile (URI):    {nprofile_uri}")

    # bech32 nprofile
    nprofile_parse = Nip21.parse(nprofile_uri)
    if nprofile_parse.as_enum().is_profile():
        nprofile_bech32 = Nip19Profile.from_nostr_uri(nprofile_uri).to_bech32()
        print(f" Profile (bech32): {nprofile_bech32}")
    # ANCHOR_END: nprofile

    print()

    # ANCHOR: nevent
    relays = ["wss://relay.damus.io"]
    nevent = Nip19Event(event.id(), keys.public_key(), kind=None, relays=relays)

    # URI nevent
    nevent_uri = nevent.to_nostr_uri()
    print(f" Event (URI):    {nevent_uri}")

    # bech32 nevent
    nevent_parse = Nip21.parse(nevent_uri)
    if nevent_parse.as_enum().is_event():
        nevent_bech32 = Nip19Event.from_nostr_uri(nevent_uri).to_bech32()
        print(f" Event (bech32): {nevent_bech32}")
    # ANCHOR_END: nevent

    print()

    # ANCHOR: naddr
    coord = Coordinate(Kind(0), keys.public_key())

    # URI naddr
    coord_uri = coord.to_nostr_uri()
    print(f" Coordinate (URI):    {coord_uri}")

    # bech32 naddr
    coord_parse = Nip21.parse(coord_uri)
    if coord_parse.as_enum().is_coord():
        coord_bech32 = Coordinate.from_nostr_uri(coord_uri).to_bech32()
        print(f" Coordinate (bech32): {coord_bech32}")
    # ANCHOR_END: naddr
