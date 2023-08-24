# Nostr
	
## Description

JavaScript bindings of the [`nostr`](https://crates.io/crates/nostr) crate.

This library - should - work on every JavaScript environment (nodejs, web, react native, ...).

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

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com
