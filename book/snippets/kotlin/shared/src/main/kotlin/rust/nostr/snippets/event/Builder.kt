package rust.nostr.snippets

// ANCHOR: full
import rust.nostr.sdk.Event
import rust.nostr.sdk.EventBuilder
import rust.nostr.sdk.Keys
import rust.nostr.sdk.Kind
import rust.nostr.sdk.NostrSigner
import rust.nostr.sdk.Tag
import rust.nostr.sdk.Timestamp

suspend fun signAndPrint(signer: NostrSigner, builder: EventBuilder) {
    // ANCHOR: sign
    val event: Event = builder.sign(signer)
    // ANCHOR_END: sign

    print(event.asJson())
}

suspend fun builder() {
    val keys = Keys.generate()
    val signer = NostrSigner.keys(keys)

    // ANCHOR: standard
    val builder1 = EventBuilder.textNote("Hello")
    // ANCHOR_END: standard

    signAndPrint(signer, builder1)

    // ANCHOR: std-custom
    val tag = Tag.alt("POW text-note")
    val builder2 =
        EventBuilder.textNote("Hello with POW")
            .tags(listOf(tag))
            .pow(20u)
            .customCreatedAt(Timestamp.fromSecs(1737976769u))
    // ANCHOR_END: std-custom

    signAndPrint(signer, builder2)

    // ANCHOR: custom
    val kind = Kind(33001u)
    val builder3 =EventBuilder(kind = kind, content = "My custom event")
    // ANCHOR_END: custom

    signAndPrint(signer, builder3)
}

// ANCHOR_END: full
