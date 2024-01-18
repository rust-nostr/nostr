# Nostr
	
## Description

JavaScript bindings of [nostr](https://github.com/rust-nostr/nostr) library.

If you're writing a typical Nostr client or bot, you may be interested in [nostr-sdk](https://www.npmjs.com/package/@rust-nostr/nostr-sdk).

This library **should** work on every JavaScript environment (nodejs, web, react native, ...).

## Getting started

```sh
npm i @rust-nostr/nostr
```
    
```javascript
const { Keys, loadWasmAsync } = require("@rust-nostr/nostr");

async function main() {
    // Load WASM 
    // if you are in a non async context, use loadWasmSync()
    await loadWasmAsync();

    // Generate random keys
    let keys = Keys.generate();

    // Hex keys
    console.log("Public key (hex): ", keys.publicKey().toHex());
    console.log("Secret key (hex): ", keys.secretKey().toHex());

    // Bech32 keys
    console.log("Public key (bech32): ", keys.publicKey().toBech32());
    console.log("Secret key (bech32): ", keys.secretKey().toBech32());
}

main();
```

More examples can be found in the [examples](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-js/examples) directory.

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips> (the Js library could be out of sync with the supported NIPs in the `nostr` rust crate)

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details
