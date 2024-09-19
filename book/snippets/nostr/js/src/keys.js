const { Keys, SecretKey, PublicKey, loadWasmSync } = require("@rust-nostr/nostr");

// ANCHOR: generate
function generate() {
    // Load WASM
    loadWasmSync();

    // Generate new random keys
    let keys = Keys.generate();
    console.log("Public key (hex): ", keys.publicKey.toHex());
    console.log("Secret key (hex): ", keys.secretKey.toHex());

    console.log("Public key (bech32): ", keys.publicKey.toBech32());
    console.log("Secret key (bech32): ", keys.secretKey.toBech32());
}
// ANCHOR_END: generate

// ANCHOR: restore
function restore() {
    // Load WASM
    loadWasmSync();

    // Parse Keys directly from secret key
    let keys1 = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");

    // Parse secret key and construct keys
    let secretKey = SecretKey.fromBech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");
    let keys2 = new Keys(secretKey);
    console.log("Secret key (hex): ", keys2.secretKey.toHex());
}
// ANCHOR_END: restore

// ANCHOR: vanity
function vanity() {
    // Load WASM
    loadWasmSync();

    // NOTE: NOT SUPPORTED YET!

    // Generate vanity keys
    // let keys = Keys.vanity(["yuk0"], true, 1);
    // console.log("Public key (bech32): ", keys.publicKey.toBech32());
    // console.log("Secret key (bech32): ", keys.secretKey.toBech32());
}
// ANCHOR_END: vanity

module.exports.vanity = vanity;
module.exports.generate = generate;
module.exports.restore = restore;
