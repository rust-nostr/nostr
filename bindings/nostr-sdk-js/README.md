# Nostr SDK
	
## Description

Nostr protocol implementation, Relay, RelayPool, high-level client library, NWC client and more.

This library **should** work on every JavaScript environment (nodejs, web, react native, ...).

## Getting started

```sh
npm i @rust-nostr/nostr-sdk
```
    
```javascript
const { Client, ClientBuilder, NostrSigner, Keys, Nip07Signer, Metadata, ZapDetails, ZapEntity, ZapType, PublicKey, loadWasmAsync } = require("@rust-nostr/nostr-sdk");

async function main() {
    // Load WASM 
    // if you are in a non async context, use loadWasmSync()
    await loadWasmAsync();

    // Compose client with private key
    let keys = Keys.generate(); // Random keys
    let signer = NostrSigner.keys(keys);
    let client = new Client(signer);

    // Compose client with NIP07 signer and WebLN zapper
    let nip07_signer = new Nip07Signer();
    let signer = NostrSigner.nip07(nip07_signer);
    let zapper = NostrZapper.webln(); // To use NWC: NostrZapper.nwc(uri); 
    let client = new ClientBuilder().signer(signer).zapper(zapper).build();

    // Add relays
    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");

    await client.connect();

    let metadata = new Metadata()
        .name("username")
        .displayName("My Username")
        .about("Description")
        .picture("https://example.com/avatar.png")
        .banner("https://example.com/banner.png")
        .nip05("username@example.com")
        .lud16("pay@yukikishimoto.com");
    
    await client.setMetadata(metadata);

    // Publish text note
    let textNoteBuilder = EventBuilder.textNote("My first text note from rust-nostr!", []);
    await client.sendEventBuilder(textNoteBuilder);

    // Compose and publish custom event (automatically signed with `NostrSigner`)
    let builder = new EventBuilder(1111, "My custom event signer with the NostrSigner", []);
    await client.sendEventBuilder(builder);

    // Send a Zap non-zap (no zap recepit created)
    let publicKey = PublicKey.fromBech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");
    let entity = ZapEntity.publicKey(publicKey);
    await client.zap(entity, 1000);

    // Send public zap
    let entity = ZapEntity.publicKey(publicKey);
    let details = new ZapDetails(ZapType.Public).message("Zap for Rust Nostr!");
    await client.zap(entity, 1000, details);
}

main();
```

More examples can be found [here](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-js/examples).

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## Book

Learn more about `rust-nostr` at <https://rust-nostr.org>.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details
