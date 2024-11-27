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
    client.sendEventBuilder(builder)
    // ANCHOR_END: publish

    // ANCHOR: output
    // TODO
    // ANCHOR_END: output
}
// ANCHOR_END: full
