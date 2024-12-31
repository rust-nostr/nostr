// ANCHOR: full
import Foundation
import NostrSDK

func hello() async throws {
    // ANCHOR: client
    let keys = Keys.generate()
    let signer = NostrSigner.keys(keys: keys)
    let client = Client(signer: signer)
    // ANCHOR_END: client

    // ANCHOR: connect
    try await client.addRelay(url: "wss://relay.damus.io")
    await client.connect()
    // ANCHOR_END: connect

    // ANCHOR: publish
    let builder = EventBuilder.textNote(content: "Hello, rust-nostr!")
    let output = try await client.sendEventBuilder(builder: builder)
    // ANCHOR_END: publish

    // ANCHOR: output
    print("Event ID: \(try output.id.toBech32())")
    print("Sent to: \(output.success)")
    print("Not sent to: \(output.failed)")
    // ANCHOR_END: output
}
// ANCHOR_END: full
