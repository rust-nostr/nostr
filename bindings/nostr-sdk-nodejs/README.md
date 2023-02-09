# Nostr SDK
	
## Description

NodeJS bindings of the [`nostr-sdk`](../../crates/nostr-sdk/) crate.

## Getting started

```sh
npm i @rust-nostr/nostr-sdk
```
    
```javascript
const { Keys } = require("@rust-nostr/nostr-sdk");

async function main() {
     let keys = Keys.generate();
    
    // Hex keys
    console.log("Public key (hex): ", keys.publicKey());
    console.log("Secret key (hex): ", keys.secretKey());
    
    // Bech32 keys
    console.log("Public key (bech32): ", keys.publicKeyBech32());
    console.log("Secret key (bech32): ", keys.secretKeyBech32());
}

main();
```

## Supported NIPs

Look at https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details

## Donations

⚡ Tips: https://getalby.com/p/yuki

⚡ Lightning Address: yuki@getalby.com
