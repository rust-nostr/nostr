import { EventId, Keys, Timestamp, Kind, EventBuilder } from "@rust-nostr/nostr-sdk"

export function eventID() {

    // Generate Keys
    const keys = Keys.generate();

    console.log();
    console.log("Event ID:");

    // ANCHOR: build-event-id
    console.log("  Build Event ID:");
    let event_id = new EventId(keys.publicKey, Timestamp.now(), new Kind(1), [], "");
    console.log(`     - ${event_id}`);
    // ANCHOR_END: build-event-id

    console.log();
    // ANCHOR: format-parse-hex
    // To Hex and then Parse
    console.log("  Event ID (hex):");
    let event_id_hex = event_id.toHex();
    console.log(`     - Hex: ${event_id_hex}`);
    console.log(`     - Parse: ${EventId.parse(event_id_hex)}`);
   // ANCHOR_END: format-parse-hex

    console.log();
    // ANCHOR: format-parse-bech32
    // To Bech32 and then Parse
    console.log("  Event ID (bech32):");
    let event_id_bech32 = event_id.toBech32();
    console.log(`     - Bech32: ${event_id_bech32}`);
    console.log(`     - Parse: ${EventId.parse(event_id_bech32)}`);
    // ANCHOR_END: format-parse-bech32

    console.log();
    // ANCHOR: format-parse-nostr-uri
    // To Nostr URI and then Parse
    console.log("  Event ID (nostr uri):");
    let event_id_nostr_uri = event_id.toNostrUri();
    console.log(`     - Nostr URI: ${event_id_nostr_uri}`);
    // UNCOMMENT_ON_RELEASE: console.log(`     - Parse: ${EventId.parse(event_id_nostr_uri)}`);
    // ANCHOR_END: format-parse-nostr-uri

    console.log();
    // ANCHOR: format-parse-bytes
    // As Bytes and then Parse
    console.log("  Event ID (bytes):");
    let event_id_bytes = event_id.asBytes();
    console.log(`     - Bytes: ${event_id_bytes}`);
    // UNCOMMENT_ON_RELEASE: console.log(`     - From Bytes: ${EventId.fromBytes(event_id_hex)}`);
    // ANCHOR_END: format-parse-bytes

    console.log();
    // ANCHOR: access-verify
    // Event ID from Event & Verfiy
    console.log("  Event ID from Event:");
    let event = EventBuilder.textNote("This is a note").signWithKeys(keys);
    console.log(`     - Event ID: ${event.id.toBech32()}`);
    console.log(`     - Verify the ID & Signature: ${event.verify()}`);
    console.log(`     - Verify the ID Only: ${event.verifyId()}`);
    // ANCHOR_END: access-verify
}
