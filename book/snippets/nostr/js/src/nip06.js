const { loadWasmSync, Keys} = require("@rust-nostr/nostr-sdk");
const { generateMnemonic } = require("bip39");

function run() {
    // Load WASM
    loadWasmSync();

    console.log();
    // ANCHOR: keys-from-seed24
    // Generate random Seed Phrase (24 words e.g. 256 bits entropy)
    let words256 = generateMnemonic(256);
    console.log("Generated Random Seed Phrase and Derived Keys:");
    console.log(`\t - Seed Words (24): ${words256}`);
    let passphrase256 = "";

    // Use Seed Phrase to generate basic Nostr keys
    let keys256 = Keys.fromMnemonic(words256, passphrase256);

    // Print Results
    console.log(`\t - Private (hex)  : ${keys256.secretKey.toHex()}`);
    console.log(`\t - Private (nsec) : ${keys256.secretKey.toBech32()}`);
    console.log(`\t - Public (hex)   : ${keys256.publicKey.toHex()}`);
    console.log(`\t - Public (npub)  : ${keys256.publicKey.toBech32()}`);
    // ANCHOR_END: keys-from-seed24


    console.log();
    // ANCHOR: keys-from-seed12
    // Generate random Seed Phrase (12 words e.g. 128 bits entropy)
    let words128 = generateMnemonic(128);
    console.log("Generated Random Seed Phrase and Derived Keys:");
    console.log(`\t - Seed Words (12): ${words128}`);
    let passphrase128 = "";

    // Use Seed Phrase to generate basic Nostr keys
    let keys128 = Keys.fromMnemonic(words128, passphrase128);

    // Print Results
    console.log(`\t - Private (hex)  : ${keys128.secretKey.toHex()}`);
    console.log(`\t - Private (nsec) : ${keys128.secretKey.toBech32()}`);
    console.log(`\t - Public (hex)   : ${keys128.publicKey.toHex()}`);
    console.log(`\t - Public (npub)  : ${keys128.publicKey.toBech32()}`);
    // ANCHOR_END: keys-from-seed12

    console.log();
    // ANCHOR: keys-from-seed-accounts
    // Advanced (with accounts) from the same wordlist
    let words = "leader monkey parrot ring guide accident before fence cannon height naive bean";
    let passphrase  = "";
    console.log("Generated Accounts:");
    console.log(`\t - Seed Words (12): ${words}`);

    // Use Seed Phrase and account to multiple Nostr keys
    for (let account = 0; account < 6; account++) {
        let nsec = Keys.fromMnemonic(words, passphrase, account).secretKey.toBech32();
        console.log(`\t - Private (nsec) Account #${account}: ${nsec}`);
    }
    // ANCHOR_END: keys-from-seed-accounts


    console.log();
    // ANCHOR: keys-from-seed-accounts-pass
    // Advanced (with accounts) from the same wordlist with in inclusion of PassPhrase
    words = "leader monkey parrot ring guide accident before fence cannon height naive bean";
    passphrase = "RustNostr";
    console.log("Generated Accounts:");
    console.log(`\t - Seed Words (12): ${words}`);

    // Use Seed Phrase, passphrase and account to multiple Nostr keys
    for (let account = 0; account < 6; account++) {
        let nsec = Keys.fromMnemonic(words, passphrase, account).secretKey.toBech32();
        console.log(`\t - Private (nsec) Account #${account}: ${nsec}`);
    }
    // ANCHOR_END: keys-from-seed-accounts-pass
}

module.exports.run = run;
