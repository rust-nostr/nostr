// ANCHOR: full
import Foundation
import NostrSDK

func encrypt() throws {
    // ANCHOR: parse-secret-key
    let secretKey = try SecretKey.parse(secretKey: "3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683")
    // ANCHOR_END: parse-secret-key

    // ANCHOR: encrypt-default
    let password = "nostr"
    let encrypted = try secretKey.encrypt(password: password)
    // ANCHOR_END: encrypt-default

    print("Encrypted secret key: \(try encrypted.toBech32())")

    // ANCHOR: encrypt-custom
    let encryptedCustom = try EncryptedSecretKey(secretKey: secretKey, password: password, logN: 12, keySecurity: KeySecurity.weak)
    // ANCHOR_END: encrypt-custom

    print("Encrypted secret key (custom): \(try encryptedCustom.toBech32())")
}

func decrypt() throws {
    // ANCHOR: parse-ncryptsec
    let encrypted = try EncryptedSecretKey.fromBech32(bech32: "ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p")
    // ANCHOR_END: parse-ncryptsec

    // ANCHOR: decrypt
    let secretKey = try encrypted.toSecretKey(password: "nostr")
    // ANCHOR_END: decrypt

    print("Decrypted secret key: \(try secretKey.toBech32())")
}
// ANCHOR_END: full
