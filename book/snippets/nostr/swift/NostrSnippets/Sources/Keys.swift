import NostrSDK
import Foundation

func keys() {
    // ANCHOR: generate
    let keys = Keys.generate()
    print("Public key (hex): \(keys.publicKey.toHex())")
    print("Secret key (hex): \(keys.secretKey.toHex())")

    print("Public key (bech32): \(keys.publicKey.toBech32())")
    print("Secret key (bech32): \(keys.secretKey.toBech32())")
    // ANCHOR_END: generate
}
