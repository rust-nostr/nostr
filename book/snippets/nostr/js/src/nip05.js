const { loadWasmSync, PublicKey, Metadata, verifyNip05, } = require("@rust-nostr/nostr");

function run() {
    // Load WASM
    loadWasmSync();

    console.log();
    // ANCHOR: set-metadata
    // Create metadata object with name and NIP05
    let metadata = new Metadata()
        .name("TestName")
        .nip05("TestName@rustNostr.com");
    // ANCHOR_END: set-metadata

    console.log();
    // ANCHOR: verify-nip05
    console.log("Verify NIP-05:");
    let nip05 = "Rydal@gitlurker.info";
    let publicKey = PublicKey.parse("npub1zwnx29tj2lnem8wvjcx7avm8l4unswlz6zatk0vxzeu62uqagcash7fhrf");
    let proxy = null;
    if (verifyNip05(publicKey, nip05, proxy)) {
        console.log(`     '${nip05}' verified, for ${publicKey.toBech32()}`);
    } else {
        console.log(`     Unable to verify NIP-05, for ${publicKey.toBech32()}`);
    };
    // ANCHOR_END: verify-nip05
}

module.exports.run = run;