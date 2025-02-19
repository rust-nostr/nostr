// ANCHOR: full
import {EncryptedSecretKey, KeySecurity, loadWasmSync, SecretKey} from "@rust-nostr/nostr-sdk";

function encrypt() {
    // ANCHOR: parse-secret-key
    let secretKey: SecretKey = SecretKey.parse("3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683");
    // ANCHOR_END: parse-secret-key

    // ANCHOR: encrypt-default
    let password: string = "nostr";
    let encrypted: EncryptedSecretKey = secretKey.encrypt(password);
    // ANCHOR_END: encrypt-default

    console.log("Encrypted secret key:", encrypted.toBech32());

    // ANCHOR: encrypt-custom
    let encryptedCustom: EncryptedSecretKey = new EncryptedSecretKey(secretKey, password, 12, KeySecurity.Weak);
    // ANCHOR_END: encrypt-custom

    console.log("Encrypted secret key (custom):", encryptedCustom.toBech32());
}

function decrypt() {
    // ANCHOR: parse-ncryptsec
    let encrypted: EncryptedSecretKey = EncryptedSecretKey.fromBech32("ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p");
    // ANCHOR_END: parse-ncryptsec

    // ANCHOR: decrypt
    let secretKey: SecretKey = encrypted.toSecretKey("nostr");
    // ANCHOR_END: decrypt

    console.log("Decrypted secret key:", secretKey.toBech32());
}

loadWasmSync();
encrypt();
decrypt();
// ANCHOR_END: full
