// ANCHOR: full
import {Keys, Client, EventBuilder, NostrSigner} from "@rust-nostr/nostr-sdk";

export async function hello() {
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
    let builder = EventBuilder.textNote("Hello, rust-nostr!", []);
    await client.sendEventBuilder(builder);
    // ANCHOR_END: publish
}
// ANCHOR_END: full
