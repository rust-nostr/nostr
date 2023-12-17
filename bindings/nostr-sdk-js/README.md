# Nostr SDK
	
## Description

A high-level, [Nostr](https://github.com/nostr-protocol/nostr) client library.

If you're writing a typical Nostr client or bot, this is likely the crate you need.

However, the crate is designed in a modular way and depends on several other lower-level libraries. If you're attempting something more custom, you might be interested in these:

- [`nostr`](https://www.npmjs.com/package/@rust-nostr/nostr): Implementation of Nostr protocol

This library **should** work on every JavaScript environment (nodejs, web, react native, ...).

## Getting started

```sh
npm i @rust-nostr/nostr-sdk
```
    
```javascript
const { Keys, Client, Metadata, EventId, PublicKey, EventBuilder, loadWasmAsync } = require("@rust-nostr/nostr-sdk");

async function main() {
    // Load WASM 
    // if you are in a non async context, use loadWasmSync()
    await loadWasmAsync();

    // Generate random keys
    let keys = Keys.generate();

    // Hex keys
    console.log("Public key (hex): ", keys.publicKey.toHex());
    console.log("Secret key (hex): ", keys.secretKey.toHex());

    // Bech32 keys
    console.log("Public key (bech32): ", keys.publicKey.toBech32());
    console.log("Secret key (bech32): ", keys.secretKey.toBech32());

    let client = new Client(keys);
    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nostr.oxtr.dev");
    await client.addRelay("wss://nostr.bitcoiner.social");
    await client.addRelay("wss://nostr.openchain.fr");

    await client.connect();

    let metadata = new Metadata()
        .name("username")
        .displayName("My Username")
        .about("Description")
        .picture("https://example.com/avatar.png")
        .banner("https://example.com/banner.png")
        .nip05("username@example.com")
        .lud16("yuki@getalby.com");
    
    await client.setMetadata(metadata);

    await client.publishTextNote("My first text note from Nostr SDK!", []);

    // Send custom event
    let event_id = EventId.fromBech32("note1z3lwphdc7gdf6n0y4vaaa0x7ck778kg638lk0nqv2yd343qda78sf69t6r");
    let public_key = PublicKey.fromBech32("npub14rnkcwkw0q5lnmjye7ffxvy7yxscyjl3u4mrr5qxsks76zctmz3qvuftjz");
    let event = EventBuilder.newReaction(event_id, public_key, "🧡").toEvent(keys);

    // Send custom event to all relays
    await client.sendEvent(event);

    // Send custom event to a specific previously added relay
    // await client.sendEventTo("wss://relay.damus.io", event);
}

main();
```

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com
