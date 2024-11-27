package rust.nostr.snippets

import rust.nostr.sdk.*

// ANCHOR: generate
fun generate() {
    val keys = Keys.generate()

    val publicKey = keys.publicKey()
    val secretKey = keys.secretKey()

    println("Public key (hex): ${publicKey.toHex()}")
    println("Secret key (hex): ${secretKey.toHex()}")

    println("Public key (bech32): ${publicKey.toBech32()}")
    println("Secret key (bech32): ${secretKey.toBech32()}")
}
// ANCHOR_END: generate

// ANCHOR: restore
fun restore() {
    // Parse secret key
    var keys = Keys.parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")

    // Parse from hex
    var secretKey = SecretKey.fromHex("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
    keys = Keys(secretKey = secretKey)

    // Parse from bech32
    secretKey = SecretKey.fromBech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
    keys = Keys(secretKey = secretKey)
}
// ANCHOR_END: restore

// ANCHOR: vanity
fun vanity() {
    val keys = Keys.vanity(listOf("yuk0"), true, 4u)

    println("Public key: ${keys.publicKey().toBech32()}")
    println("Secret key: ${keys.secretKey().toBech32()}")
}
// ANCHOR_END: vanity
