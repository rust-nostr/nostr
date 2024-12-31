import {Keys, PublicKey, nip44Encrypt, nip44Decrypt, NIP44Version} from "@rust-nostr/nostr-sdk";

export function run() {
    let keys = Keys.generate();

    let public_key = PublicKey.parse("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

    let ciphertext = nip44Encrypt(keys.secretKey, public_key, "my message", NIP44Version.V2)
    console.log("Encrypted: " + ciphertext)

    let plaintext = nip44Decrypt(keys.secretKey, public_key, ciphertext)
    console.log("Decrypted: " + plaintext)
}
