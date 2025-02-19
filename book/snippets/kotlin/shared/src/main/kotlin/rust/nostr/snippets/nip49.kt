package rust.nostr.snippets

// ANCHOR: full
import rust.nostr.sdk.*

fun encrypt() {
    // ANCHOR: parse-secret-key
    val secretKey: SecretKey = SecretKey.parse("3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683")
    // ANCHOR_END: parse-secret-key

    // ANCHOR: encrypt-default
    val password = "nostr"
    val encrypted: EncryptedSecretKey = secretKey.encrypt(password)
    // ANCHOR_END: encrypt-default

    println("Encrypted secret key: ${encrypted.toBech32()}")

    // ANCHOR: encrypt-custom
    val encryptedCustom = EncryptedSecretKey(secretKey, password, 12u, KeySecurity.WEAK)
    // ANCHOR_END: encrypt-custom

    println("Encrypted secret key (custom): ${encryptedCustom.toBech32()}")
}

fun decrypt() {
    // ANCHOR: parse-ncryptsec
    val encrypted: EncryptedSecretKey = EncryptedSecretKey.fromBech32("ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p")
    // ANCHOR_END: parse-ncryptsec

    // ANCHOR: decrypt
    val secretKey: SecretKey = encrypted.toSecretKey(password = "nostr")
    // ANCHOR_END: decrypt

    println("Decrypted secret key: ${secretKey.toBech32()}")
}

fun main() {
    encrypt()
    decrypt()
}
// ANCHOR_END: full
