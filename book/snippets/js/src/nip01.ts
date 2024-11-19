import { Keys, Metadata, EventBuilder } from "@rust-nostr/nostr-sdk";

export function run() {
    // Generate random keys
    let keys = Keys.generate();

    console.log();
    // ANCHOR: create-event
    // Create metadata object with desired content
    let metadataContent = new Metadata()
        .name("TestName")
        .displayName("JsTestur")
        .about("This is a Test Account for Rust Nostr JS Bindings")
        .website("https://rust-nostr.org/")
        .picture("https://avatars.githubusercontent.com/u/123304603?s=200&v=4")
        .banner("https://nostr-resources.com/assets/images/cover.png")
        .nip05("TestName@rustNostr.com");

    // Build metadata event and assign content
    let builder = EventBuilder.metadata(metadataContent);

    // Signed event and print details
    console.log("Creating Metadata Event:");
    let event = builder.signWithKeys(keys);

    console.log(" Event Details:");
    console.log(`     Author    : ${event.author.toBech32()}`);
    console.log(`     Kind      : ${event.kind.valueOf()}`);
    console.log(`     Content   : ${event.content.toString()}`);
    console.log(`     Datetime  : ${event.createdAt.toHumanDatetime()}`);
    console.log(`     Signature : ${event.signature.toString()}`);
    console.log(`     Verify    : ${event.verify()}`);
    console.log(`     JSON      : ${event.asJson()}`);
    // ANCHOR_END: create-event

    console.log();
    // ANCHOR: create-metadata
    // Deserialize Metadata from event
    console.log("Deserializing Metadata Event:");
    let metadata = Metadata.fromJson(event.content);

    console.log(" Metadata Details:");
    console.log(`     Name      : ${metadata.getName()}`);
    console.log(`     Display   : ${metadata.getDisplayName()}`);
    console.log(`     About     : ${metadata.getAbout()}`);
    console.log(`     Website   : ${metadata.getWebsite()}`);
    console.log(`     Picture   : ${metadata.getPicture()}`);
    console.log(`     Banner    : ${metadata.getBanner()}`);
    console.log(`     NIP05     : ${metadata.getNip05()}`);
    // ANCHOR_END: create-metadata
}
