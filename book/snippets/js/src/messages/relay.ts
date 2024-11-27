import { RelayMessage, EventBuilder, Keys } from "@rust-nostr/nostr-sdk";

export async function run() {
    const keys = Keys.generate();
    const event = EventBuilder.textNote("TestTextNoTe").signWithKeys(keys);

    console.log("\nRelay Messages:");

    // ANCHOR: event-message
    console.log("  Event Relay Message:");
    let relayMessage = RelayMessage.event("subscription_ID_abc123", event);
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: event-message

    console.log();
    // ANCHOR: ok-message
    console.log("  Event Acceptance Relay Message:");
    relayMessage = RelayMessage.ok(event.id, false, "You have no power here, Gandalf The Grey");
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: ok-message

    console.log();
    // ANCHOR: eose-message
    console.log("  End of Stored Events Relay Message:");
    relayMessage = RelayMessage.eose("subscription_ID_abc123");
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: eose-message

    console.log();
    // ANCHOR: closed-message
    console.log("  Closed Relay Message:");
    relayMessage = RelayMessage.closed("subscription_ID_abc123", "So long and thanks for all the fish");
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: closed-message

    console.log();
    // ANCHOR: notice-message
    console.log("  Notice Relay Message:");
    relayMessage = RelayMessage.notice("You have been served");
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: notice-message

    console.log();
    // ANCHOR: parse-message
    console.log("  Parse Relay Message:");
    relayMessage = RelayMessage.fromJson('["NOTICE","You have been served"]');
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: parse-message

    console.log();
    // ANCHOR: auth-message
    console.log("  Auth Relay Message:");
    relayMessage = RelayMessage.auth("I Challenge You To A Duel! (or some other challenge string)");
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: auth-message

    console.log();
    // ANCHOR: count-message
    console.log("  Count Relay Message:");
    relayMessage = RelayMessage.count("subscription_ID_abc123", 42);
    console.log(`     - JSON: ${relayMessage.asJson()}`);
    // ANCHOR_END: count-message
}
