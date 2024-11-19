import { Keys } from "@rust-nostr/nostr-sdk"

// ANCHOR: generate
export function generate() {
    // Generate new random keys
    let keys = Keys.generate();
    console.log("Public key (hex): ", keys.publicKey.toHex());
    console.log("Secret key (hex): ", keys.secretKey.toHex());

    console.log("Public key (bech32): ", keys.publicKey.toBech32());
    console.log("Secret key (bech32): ", keys.secretKey.toBech32());
}
// ANCHOR_END: generate

// ANCHOR: restore
export function restore() {
    // Parse directly from secret key
    let keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");
    console.log("Secret key (hex): ", keys.secretKey.toHex());
}
// ANCHOR_END: restore

// ANCHOR: vanity
export function vanity() {
    // NOTE: NOT SUPPORTED YET!

    // Generate vanity keys
    // let keys = Keys.vanity(["yuk0"], true, 1);
    // console.log("Public key (bech32): ", keys.publicKey.toBech32());
    // console.log("Secret key (bech32): ", keys.secretKey.toBech32());
}
// ANCHOR_END: vanity
