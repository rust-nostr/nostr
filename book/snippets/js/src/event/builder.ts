// ANCHOR: full
import {Keys, EventBuilder, Tag, Timestamp, Kind, loadWasmSync, NostrSigner} from "@rust-nostr/nostr-sdk"

async function signAndPrint(signer: NostrSigner, builder: EventBuilder) {
    // ANCHOR: sign
    let event = await builder.sign(signer);
    // ANCHOR_END: sign

    console.log(event.asJson())
}

async function run() {
    // Load WASM
    loadWasmSync();

    let keys = Keys.generate();
    let signer = NostrSigner.keys(keys);

    // ANCHOR: standard
    let builder1 = EventBuilder.textNote("Hello");
    // ANCHOR_END: standard

    await signAndPrint(signer, builder1);

    // ANCHOR: std-custom
    let builder2 =
        EventBuilder.textNote("Hello with POW")
            .tags([Tag.alt("POW text-note")])
            .pow(20)
            .customCreatedAt(Timestamp.fromSecs(1737976769));
    // ANCHOR_END: std-custom

    await signAndPrint(signer, builder2);

    // ANCHOR: custom
    let kind = new Kind(33001);
    let builder3 = new EventBuilder(kind, "My custom event");
    // ANCHOR_END: custom

    await signAndPrint(signer, builder3);
}

run();
// ANCHOR_END: full
