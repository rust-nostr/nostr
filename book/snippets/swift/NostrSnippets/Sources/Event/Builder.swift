// ANCHOR: full
import NostrSDK
import Foundation

func signAndPrint(signer: NostrSigner, builder: EventBuilder) async throws {
    // ANCHOR: sign
    let event = try await builder.sign(signer: signer)
    // ANCHOR_END: sign

    print(try event.asJson())
}

func builder() async throws{
    let keys = Keys.generate()
    let signer = NostrSigner.keys(keys: keys)

    // ANCHOR: standard
    let builder1 = EventBuilder.textNote(content: "Hello")
    // ANCHOR_END: standard

    try await signAndPrint(signer: signer, builder: builder1)

    // ANCHOR: std-custom
    let tag = Tag.alt(summary: "POW text-note")
    let timestamp = Timestamp.fromSecs(secs: 1737976769)
    let builder2 = EventBuilder.textNote(content: "Hello with POW")
        .tags(tags: [tag])
        .pow(difficulty: 20)
        .customCreatedAt(createdAt: timestamp)

    // ANCHOR_END: std-custom

    try await signAndPrint(signer: signer, builder: builder2)

    // ANCHOR: custom
    let kind = Kind(kind: 33001)
    let builder3 = EventBuilder(kind: kind, content: "My custom event")
    // ANCHOR_END: custom

    try await signAndPrint(signer: signer, builder: builder3)
}
// ANCHOR_END: full
