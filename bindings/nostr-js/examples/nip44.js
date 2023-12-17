const { Keys, PublicKey, nip44_encrypt, nip44_decrypt, NIP44Version, loadWasmAsync } = require("../");

async function main() {
    await loadWasmAsync();

    let keys = Keys.generate();
    
    let public_key = PublicKey.fromHex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

    let ciphertext = nip44_encrypt(keys.secretKey, public_key, "my message", NIP44Version.V2)
    console.log("Encrypted: " + ciphertext)

    let plaintext = nip44_decrypt(keys.secretKey, public_key, ciphertext)
    console.log("Decrypted: " + plaintext)
}

main();