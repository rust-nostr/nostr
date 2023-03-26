# Nostr
	
## Description

NodeJS bindings of the [`nostr`](https://crates.io/crates/nostr) crate.

This library works only on native environments (Linux, macOS and Windows).

Check also [`@rust-nostr/nostr`](https://www.npmjs.com/package/@rust-nostr/nostr) for the JavaScript bindings.

## Getting started

```sh
npm i @rust-nostr/nostr-nodejs
```

When installing, NPM will download the corresponding prebuilt Rust library for your current host system.
    
```javascript
const { Keys } = require("@rust-nostr/nostr-nodejs");

async function main() {
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

More examples can be found in the [examples](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-nodejs/examples) directory.

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com
