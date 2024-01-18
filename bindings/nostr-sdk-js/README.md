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
const { Client, ClientSigner, Keys, Nip07Signer, Metadata, loadWasmAsync } = require("@rust-nostr/nostr-sdk");

async function main() {
    // Load WASM 
    // if you are in a non async context, use loadWasmSync()
    await loadWasmAsync();

    // Compose client with private key
    let keys = Keys.generate(); // Random keys
    let signer = ClientSigner.keys(keys);
    let client = new Client(signer);

    // Compose client with NIP07 signer
    let nip07_signer = new Nip07Signer();
    let signer = ClientSigner.nip07(nip07_signer);
    let client = new Client(signer);

    // Add relays
    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nostr.oxtr.dev");
    await client.addRelay("wss://nostr.bitcoiner.social");
    await client.addRelay("wss://nostr.openchain.fr");

    // Add multiple relays at once
    await client.addRelays([
        "wss://relay.damus.io",
        "wss://nostr.oxtr.dev",
    ]);

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

    // Publish text note
    await client.publishTextNote("My first text note from Nostr SDK!", []);

    // Compose and publish custom event (automatically signed with `ClientSigner`)
    let builder = new EventBuilder(1111, "My custom event signer with the ClientSigner", []);
    await client.sendEventBuilder(builder);
}

main();
```

More examples can be found at:

* https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-js/examples
* https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-js/examples

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details
