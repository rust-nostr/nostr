const { ClientMessage, EventBuilder, Filter, Keys, loadWasmAsync } = require('@rust-nostr/nostr');

async function run() {
    await loadWasmAsync();

    const keys = Keys.generate();
    const event = EventBuilder.textNote("TestTextNoTe", []).toEvent(keys);

    console.log()
    console.log("Client Messages:");

    // ANCHOR: event-message
    // Create Event client message
    console.log("  Event Client Message:");
    let clientMessage = ClientMessage.event(event);
    console.log(`     - JSON: ${clientMessage.asJson()}`);
    // ANCHOR_END: event-message

    console.log();
    // ANCHOR: req-message
    // Create Request client message
    console.log("  Request Client Message:");
    let f = new Filter().id(event.id);
    clientMessage = ClientMessage.req("ABC123", [f]);
    console.log(`     - JSON: ${clientMessage.asJson()}`);
    // ANCHOR_END: req-message

    console.log();
    // ANCHOR: close-message
    // Create Close client message
    console.log("  Close Client Message:");
    clientMessage = ClientMessage.close("ABC123");
    console.log(`     - JSON: ${clientMessage.asJson()}`);
    // ANCHOR_END: close-message

    console.log();
    // ANCHOR: parse-message
    // Parse Messages from JSON
    console.log("  Parse Client Messages:");
    clientMessage = ClientMessage.fromJson('["REQ","ABC123",{"#p":["421a4dd67be773903f805bcb7975b4d3377893e0e09d7563b8972ee41031f551"]}]');
    console.log(`     - JSON: ${clientMessage.asJson()}`);
    // ANCHOR_END: parse-message

    console.log();
    // ANCHOR: auth-message
    // Create Auth client message  (NIP42)
    console.log("  Auth Client Message:");
    clientMessage = ClientMessage.auth(event);
    console.log(`     - JSON: ${clientMessage.asJson()}`);
    // ANCHOR_END: auth-message

    console.log();
    // ANCHOR: count-message
    // Create Count client message (NIP45)
    console.log("  Count Client Message:");
    f = new Filter().pubkey(keys.publicKey);
    clientMessage = ClientMessage.count("ABC123", [f]);
    console.log(`     - JSON: ${clientMessage.asJson()}`);
    // ANCHOR_END: count-message

}

module.exports.run = run;