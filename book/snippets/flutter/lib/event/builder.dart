// ANCHOR: full
import 'package:nostr_sdk/nostr_sdk.dart';

Future<void> signAndPrint({signer = NostrSigner, builder = EventBuilder}) async {
    // ANCHOR: sign
    Event event = await builder.sign(signer);
    // ANCHOR_END: sign

    print(event.asJson());
}

Future<void> event() async {
    Keys keys = Keys.generate();
    NostrSigner signer = NostrSigner.keys(keys: keys);

    // ANCHOR: standard
    EventBuilder builder1 = EventBuilder.textNote(content: "Hello");
    // ANCHOR_END: standard

    await signAndPrint(signer: signer, builder: builder1);

    // ANCHOR: std-custom
    Tag tag = Tag.parse(tag: ["client", "rust-nostr"]);
    EventBuilder builder2 =
        EventBuilder.textNote(content: "Hello with POW")
        .tag(tag: tag)
        .pow(difficulty: 20)
        .customCreatedAt(createdAt: BigInt.from(1737976769));
    // ANCHOR_END: std-custom

    await signAndPrint(signer: signer, builder: builder2);

    // ANCHOR: custom
    EventBuilder builder3 = EventBuilder(kind: 33001, content: "My custom event");
    // ANCHOR_END: custom

    await signAndPrint(signer: signer, builder: builder3);
}
// ANCHOR_END: full
