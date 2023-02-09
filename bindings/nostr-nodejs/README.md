# Nostr NodeJS
	
## Description

NodeJS bindings of the `nostr` crate.

## Getting started

Just add the latest release to your `package.json`:

```sh
npm install @rust-nostr/nostr
```
    
```javascript
const { Keys } = require("@rust-nostr/nostr");

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

