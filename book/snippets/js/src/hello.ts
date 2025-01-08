// ANCHOR: full
import {Keys, Client, EventBuilder, NostrSigner, loadWasmAsync} from "@rust-nostr/nostr-sdk";

async function hello() {
    // Load WASM
    await loadWasmAsync();

    // ANCHOR: client
    let keys: Keys = Keys.generate();
    let signer = NostrSigner.keys(keys);
    let client = new Client(signer);
    // ANCHOR_END: client

    // ANCHOR: connect
    await client.addRelay("wss://relay.damus.io")
    await client.connect();
    // ANCHOR_END: connect

    // ANCHOR: publish
    let builder = EventBuilder.textNote("Hello, rust-nostr!");
    let output = await client.sendEventBuilder(builder);
    // ANCHOR_END: publish

    // ANCHOR: output
    console.log("Event ID:", output.id.toBech32());
    console.log("Sent to:", output.success);
    console.log("Not sent to:", output.failed);
    // ANCHOR_END: output
}

hello();
// ANCHOR_END: full
