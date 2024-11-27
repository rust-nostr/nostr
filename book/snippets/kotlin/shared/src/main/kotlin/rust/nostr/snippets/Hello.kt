package rust.nostr.snippets

// ANCHOR: full
import rust.nostr.sdk.*

suspend fun hello() {
    // ANCHOR: client
    val keys = Keys.generate()
    val signer = NostrSigner.keys(keys)
    val client = Client(signer = signer)
    // ANCHOR_END: client

    // ANCHOR: connect
    client.addRelay("wss://relay.damus.io")
    client.connect()
    // ANCHOR_END: connect

    // ANCHOR: publish
    val builder = EventBuilder.textNote("Hello, rust-nostr!")
    val res = client.sendEventBuilder(builder)
    // ANCHOR_END: publish

    // ANCHOR: output
    println("Event ID: ${res.id.toBech32()}")
    println("Sent to: ${res.output.success}")
    println("Not sent to: ${res.output.failed}")
    // ANCHOR_END: output
}
// ANCHOR_END: full
