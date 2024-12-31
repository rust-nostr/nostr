import Foundation
import NostrSDK

func nip44() throws {
    let keys = Keys.generate()

    let publicKey = try PublicKey.parse(publicKey: "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")

    let ciphertext = try nip44Encrypt(secretKey: keys.secretKey(), publicKey: publicKey, content: "my message", version: Nip44Version.v2)
    print("Encrypted: \(ciphertext)");

    let plaintext = try nip44Decrypt(secretKey: keys.secretKey(), publicKey: publicKey, payload: ciphertext)
    print("Decrypted: \(plaintext)");
}
