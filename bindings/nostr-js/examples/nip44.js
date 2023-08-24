const { Keys, nip44_encrypt, nip44_decrypt, Nip44Version, loadWasmSync } = require("../");

function main() {
    loadWasmSync();

    let alice_keys = Keys.fromSkStr("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a");
    let bob_keys = Keys.fromSkStr("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99");

    let payload = nip44_encrypt(alice_keys.secretKey, bob_keys.publicKey, "hello", 1);
    console.log(payload);

    plaintext = nip44_decrypt(bob_keys.secretKey, alice_keys.publicKey, payload)
    console.log(plaintext);
}

main();