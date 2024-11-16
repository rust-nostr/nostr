import NostrSDK
import Foundation

func keys() throws {
    // ANCHOR: generate
    let keys = Keys.generate()
    let publicKey = keys.publicKey()
    let secretKey = keys.secretKey()

    print("Public key (hex): \(publicKey.toHex())")
    print("Secret key (hex): \(secretKey.toHex())")

    print("Public key (bech32): \(try publicKey.toBech32())")
    print("Secret key (bech32): \(try secretKey.toBech32())")
    // ANCHOR_END: generate
}
