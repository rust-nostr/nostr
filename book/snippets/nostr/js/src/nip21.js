const { loadWasmSync, Keys, EventBuilder, Nip19Profile, Nip19Event, Coordinate} = require("@rust-nostr/nostr");

function run(){
    // Load WASM
    loadWasmSync();

    console.log();
    console.log("Nostr URIs:");

    // ANCHOR: npub
    let keys = Keys.generate();
    // bech32 npub
    let pk_bech32 = keys.publicKey.toBech32();
    console.log(` Public key (bech32): ${pk_bech32}`);
    // ANCHOR_END: npub

    console.log();
    // ANCHOR: note
    let event = EventBuilder.textNote("Hello from Rust Nostr JS Bindings!", []).toEvent(keys);

    // bech32 note
    let note_bech32 = event.id.toBech32()
    console.log(` Event (bech32): ${note_bech32}`);
    // ANCHOR_END: note

    console.log();
    // ANCHOR: nprofile
    let relays = ["wss://relay.damus.io"];
    let nprofile = new Nip19Profile(keys.publicKey, relays);

    // URI nprofile
    let nprofile_uri = nprofile.toNostrUri();
    console.log(` Profile (URI):    ${nprofile_uri}`);

    // bech32 nprofile
    let nprofile_bech32 = Nip19Profile.fromNostrUri(nprofile_uri).toBech32();
    console.log(` Profile (bech32): ${nprofile_bech32}`);
    // ANCHOR_END: nprofile
    
    console.log();
    // ANCHOR: nevent
    let nevent = new Nip19Event(event.id, keys.publicKey, null, relays);

    // URI nevent
    let nevent_uri = nevent.toNostrUri();
    console.log(` Event (URI):    ${nevent_uri}`);

    // bech32 nevent
    let nevent_bech32 = Nip19Event.fromNostrUri(nevent_uri).toBech32();
    console.log(` Event (bech32): ${nevent_bech32}`);
    // ANCHOR_END: nevent

    console.log();
    // ANCHOR: naddr
    // URI naddr
    let coord_uri = new Coordinate(event.kind, keys.publicKey).toNostrUri();
    console.log(` Coordinate (URI):    ${coord_uri}`);

    // bech32 naddr
    let coord_bech32 = new Coordinate(event.kind, keys.publicKey).toBech32();
    console.log(` Coordinate (bech32): ${coord_bech32}`);
    // ANCHOR_END: naddr

}

run();