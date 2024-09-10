const { loadWasmSync, Keys, EventBuilder, Nip19Profile, Nip19Event, Nip19Relay} = require("@rust-nostr/nostr");

function run() {
    // Load WASM
    loadWasmSync();

    // Generate random keys
    let keys = Keys.generate();

    console.log();
    console.log("Bare keys and ids (bech32):");
    // ANCHOR: nip19-npub
    console.log(` Public key: ${keys.publicKey.toBech32()}`);
    // ANCHOR_END: nip19-npub

    // ANCHOR: nip19-nsec
    console.log(` Secret key: ${keys.secretKey.toBech32()}`);
    // ANCHOR_END: nip19-nsec

    // ANCHOR: nip19-note
    let event = EventBuilder.textNote("Hello from Rust Nostr JS Bindings!", []).toEvent(keys);
    console.log(` Event     : ${event.id.toBech32()}`);
    // ANCHOR_END: nip19-note

    console.log();
    console.log("Shareable identifiers with extra metadata (bech32):");
    // ANCHOR: nip19-nprofile-encode
    // Create NIP-19 profile including relays data
    let relays = ["wss://relay.damus.io"];
    let nprofile = new Nip19Profile(keys.publicKey, relays);
    console.log(` Profile (encoded): ${nprofile.toBech32()}`);
    // ANCHOR_END: nip19-nprofile-encode

    // ANCHOR: nip19-nprofile-decode
    // Decode NIP-19 profile
    let decode_nprofile = Nip19Profile.fromBech32(nprofile.toBech32());
    console.log(` Profile (decoded): ${decode_nprofile.publicKey().toBech32()}`);
    // ANCHOR_END: nip19-nprofile-decode

    console.log();
    // ANCHOR: nip19-nevent-encode
    // Create NIP-19 event including author and relays data
    let nevent = new Nip19Event(event.id, keys.publicKey, null, relays);
    console.log(` Event (encoded): ${nevent.toBech32()}`);
    // ANCHOR_END: nip19-nevent-encode

    // ANCHOR: nip19-nevent-decode
    // Decode NIP-19 event
    let decode_nevent = Nip19Event.fromBech32(nevent.toBech32());
    console.log(` Event (decoded): ${decode_nevent.eventId().toBech32()}`);
    // ANCHOR_END: nip19-nevent-decode

    console.log();
    // ANCHOR: nip19-nrelay-encode
    // Create NIP-19 relay
    let relay = new Nip19Relay("wss://relay.damus.io");
    console.log(` Relay (encoded): ${relay.toBech32()}`);
    // ANCHOR_END: nip19-nrelay-encode

    // ANCHOR: nip19-nrelay-decode
    // Decode NIP-19 relay
    let decode_relay = Nip19Relay.fromBech32(relay.toBech32());
    console.log(` Relay (decoded): ${decode_relay.url()}`);
    // ANCHOR_END: nip19-nrelay-decode

}

module.exports.run = run;