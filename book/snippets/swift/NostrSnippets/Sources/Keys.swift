// ANCHOR: full
import Foundation
import NostrSDK

// ANCHOR: generate
func generate() throws {
    let keys = Keys.generate()

    let publicKey = keys.publicKey()
    let secretKey = keys.secretKey()

    print("Public key (hex): \(publicKey.toHex())")
    print("Secret key (hex): \(secretKey.toHex())")

    print("Public key (bech32): \(try publicKey.toBech32())")
    print("Secret key (bech32): \(try secretKey.toBech32())")
}
// ANCHOR_END: generate

// ANCHOR: restore
func restore() throws {
    let keys = try Keys.parse(secretKey: "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")

    let publicKey = keys.publicKey()

    print("Public key: \(try publicKey.toBech32())")
}
// ANCHOR_END: restore

// ANCHOR: vanity
func vanity() throws {
    let keys = try Keys.vanity(prefixes: ["0000", "yuk", "yuk0"], bech32: true, numCores: 8)

    let publicKey = keys.publicKey()
    let secretKey = keys.secretKey()

    print("Public key: \(try publicKey.toBech32())")
    print("Secret key: \(try secretKey.toBech32())")
}
// ANCHOR_END: vanity
// ANCHOR_END: full
