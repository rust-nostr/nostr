import {Keys, EventBuilder, Tag, RelayMetadata, RelayListItem, Kind, loadWasmSync} from "@rust-nostr/nostr-sdk";

function run(){
    // Load WASM
    loadWasmSync();

    // Generate Keys
    let keys = Keys.generate();

    console.log();
    console.log("Relay Metadata:");
    // ANCHOR: relay-metadata-simple
    // Create relay list
    let relays = [
        new RelayListItem("wss://relay.damus.io", RelayMetadata.Read),
        new RelayListItem("wss://relay.primal.net", RelayMetadata.Write),
        new RelayListItem("wss://relay.nostr.band")
    ];

    // Build/sign event
    let builder = EventBuilder.relayList(relays);
    let event = builder.signWithKeys(keys);

    // Print event as json
    console.log(` Event: ${event.asJson()}`);
    // ANCHOR_END: relay-metadata-simple

    console.log();
    // ANCHOR: relay-metadata-custom
    // Create relay metadata tags
    let tag1 = Tag.relayMetadata("wss://relay.damus.io", RelayMetadata.Read);
    let tag2 = Tag.relayMetadata("wss://relay.primal.net", RelayMetadata.Write);
    let tag3 = Tag.relayMetadata("wss://relay.nostr.band");

    // Build/sign event
    let kind = new Kind(10002);
    builder = new EventBuilder(kind, "").tags([tag1, tag2, tag3]);
    event = builder.signWithKeys(keys);

    // Print event as json
    console.log(` Event: ${event.asJson()}`);
    // ANCHOR_END: relay-metadata-custom
}

run();
