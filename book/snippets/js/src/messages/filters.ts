import { Filter, Keys, Kind, EventBuilder, Timestamp, Tag } from "@rust-nostr/nostr-sdk";

export async function run() {
    // Generate keys and Events
    const keys = Keys.generate();
    const keys2 = Keys.generate();

    const kind0 = new Kind(0);
    const kind1 = new Kind(1);
    const kind4 = new Kind(4);

    const event = EventBuilder.textNote("Hello World!").signWithKeys(keys);
    const event2 = new EventBuilder(kind0, "Goodbye World!")
        .tags([Tag.identifier("Identification D Tag")])
        .signWithKeys(keys2);

    console.log();
    console.log("Creating Filters:");

    // ANCHOR: create-filter-id
    // Filter for specific ID
    console.log("  Filter for specific Event ID:");
    let f = new Filter().id(event.id);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-id

    console.log();
    // ANCHOR: create-filter-author
    // Filter for specific Author
    console.log("  Filter for specific Author:");
    f = new Filter().author(keys.publicKey);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-author

    console.log();
    // ANCHOR: create-filter-kind-pk
    // Filter by PK and Kinds
    console.log("  Filter with PK and Kinds:");
    f = new Filter()
        .pubkey(keys.publicKey)
        .kind(kind1);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-kind-pk

    console.log();
    // ANCHOR: create-filter-search
    // Filter for specific string
    console.log("  Filter for specific search string:");
    f = new Filter().search("Ask Nostr Anything");
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-search

    console.log();
    // ANCHOR: create-filter-timeframe
    console.log("  Filter for events from specific public key within given timeframe:");
    // Create timestamps
    const date = new Date(2009, 1, 3, 0, 0);
    const timestamp = Math.floor(date.getTime() / 1000);
    const sinceTs = Timestamp.fromSecs(timestamp);
    const untilTs = Timestamp.now();

    // Filter with timeframe
    f = new Filter()
        .pubkey(keys.publicKey)
        .since(sinceTs)
        .until(untilTs);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-timeframe

    console.log();
    // ANCHOR: create-filter-limit
    // Filter for specific PK with limit
    console.log("  Filter for specific Author, limited to 10 Events:");
    f = new Filter()
        .author(keys.publicKey)
        .limit(10);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-limit

    console.log();
    // ANCHOR: create-filter-hashtag
    // Filter for Hashtags
    console.log("  Filter for a list of Hashtags:");
    f = new Filter().hashtags(["#Bitcoin", "#AskNostr", "#Meme"]);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-hashtag

    console.log();
    // ANCHOR: create-filter-reference
    // Filter for Reference
    console.log("  Filter for a Reference:");
    f = new Filter().reference("This is my NIP-12 Reference");
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-reference

    console.log();
    // ANCHOR: create-filter-identifier
    // Filter for Identifier
    console.log("  Filter for a Identifier:");
    f = new Filter().identifier("This is my NIP-12 Identifier");
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: create-filter-identifier

    console.log();
    console.log("Modifying Filters:");
    // ANCHOR: modify-filter
    // Modifying Filters (adding/removing)
    f = new Filter()
        .pubkeys([keys.publicKey, keys2.publicKey])
        .ids([event.id, event2.id])
        .kinds([kind0, kind1])
        .author(keys.publicKey);

    // Add an additional Kind to existing filter
    f = f.kinds([kind4]);

    // Print Results
    console.log("  Before:");
    console.log(`     ${f.asJson()}`);
    console.log();

    // Remove PKs, Kinds and IDs from filter
    f = f.removePubkeys([keys2.publicKey]);
    console.log(" After (remove pubkeys):");
    console.log(`     ${f.asJson()}`);
    const kind_rem0 = new Kind(0);
    const kind_rem4 = new Kind(4);
    f = f.removeKinds([kind_rem0, kind_rem4]);
    console.log("  After (remove kinds):");
    console.log(`     ${f.asJson()}`);

    f = f.removeIds([event2.id]);
    console.log("  After (remove IDs):");
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: modify-filter

    console.log();
    console.log("Other Filter Operations:");
    // ANCHOR: other-parse
    // Parse filter
    console.log("  Parse Filter from Json:");
    const fJson = f.asJson();
    f = Filter.fromJson(fJson);
    console.log(`     ${f.asJson()}`);
    // ANCHOR_END: other-parse

    console.log();
    // ANCHOR: other-match
    console.log("  Logical tests:");
    const kind_match = new Kind(1);
    f = new Filter().author(keys.publicKey).kind(kind_match);
    console.log(`     Event match for filter: ${f.matchEvent(event)}`);
    console.log(`     Event2 match for filter: ${f.matchEvent(event2)}`);
    // ANCHOR_END: other-match
}
