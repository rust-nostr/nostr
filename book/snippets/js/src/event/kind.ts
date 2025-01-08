import { Kind, Keys, EventBuilder, Metadata, loadWasmSync } from "@rust-nostr/nostr-sdk"

function kind() {
    // Load WASM
    loadWasmSync();

    // Generate random keys
    let keys = Keys.generate();

    console.log();
    console.log("Kind:");

    // ANCHOR: kind-int
    console.log("  Kind by number:");
    let kind = new Kind(1);
    console.log(`     - Kind 1: ${kind.toString()}`);
    kind = new Kind(0);
    console.log(`     - Kind 0: ${kind.toString()}`);
    kind = new Kind(3);
    console.log(`     - Kind 3: ${kind.toString()}`);
    // ANCHOR_END: kind-int

    console.log();
    // ANCHOR: kind-methods
    console.log("  Kind methods EventBuilder:");
    let event = EventBuilder.textNote("This is a note").signWithKeys(keys);
    console.log(`     - Kind textNote(): ${event.kind.asU16()}`);
    event = EventBuilder.metadata(new Metadata()).signWithKeys(keys);
    console.log(`     - Kind metadata(): ${event.kind.asU16()}`);
    event = EventBuilder.contactList([]).signWithKeys(keys);
    console.log(`     - Kind contactList(): ${event.kind.asU16()}`);
    // ANCHOR_END: kind-methods

    console.log();
    // ANCHOR: kind-tests
    console.log("  Kind Logical Tests:");
    kind = new Kind(30001);
    console.log(`     - Is ${kind.toString()} addressable?: ${kind.isAddressable()}`);
    kind = new Kind(20001);
    console.log(`     - Is ${kind.toString()} ephemeral?: ${kind.isEphemeral()}`);
    kind = new Kind(5001);
    console.log(`     - Is ${kind.toString()} job request?: ${kind.isJobRequest()}`);
    kind = new Kind(6001);
    console.log(`     - Is ${kind.toString()} job result?: ${kind.isJobResult()}`);
    kind = new Kind(1);
    console.log(`     - Is ${kind.toString()} regular?: ${kind.isRegular()}`);
    kind = new Kind(10001);
    console.log(`     - Is ${kind.toString()} replaceable?: ${kind.isReplaceable()}`);
    // ANCHOR_END: kind-tests
}

kind();
