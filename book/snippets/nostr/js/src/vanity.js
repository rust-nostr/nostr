const { Keys, loadWasmSync } = require("@rust-nostr/nostr");

function vanity() {
    // Load WASM
    loadWasmSync();

    // NOTE: NOT SUPPORTED YET!

    // Generate vanity keys
    // let keys = Keys.vanity(["yuk0"], true, 1);
    // console.log("Public key (bech32): ", keys.publicKey.toBech32());
    // console.log("Secret key (bech32): ", keys.secretKey.toBech32());
}

module.exports.vanity = vanity;