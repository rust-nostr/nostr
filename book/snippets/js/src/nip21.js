const { loadWasmSync, Keys, EventBuilder, Nip19Profile, Nip19Event, Coordinate} = require("@rust-nostr/nostr-sdk");

function run(){
    // Load WASM
    loadWasmSync();

    let keys = Keys.generate();

    console.log();
    console.log("Nostr URIs:");

    // ANCHOR: npub
    let pk_uri = keys.publicKey.toNostrUri();
    console.log(` Public key (URI): ${pk_uri}`);
    // ANCHOR_END: npub

    console.log();
    // ANCHOR: note
    let event = EventBuilder.textNote("Hello from rust-nostr JS bindings!", []).signWithKeys(keys);
    let note_uri = event.id.toNostrUri()
    console.log(` Event (URI): ${note_uri}`);
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

module.exports.run = run;
