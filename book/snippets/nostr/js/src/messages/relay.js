const { loadWasmSync, RelayMessage } = require("@rust-nostr/nostr");

function relayMessageJson() {
    // Load WASM
    loadWasmSync();

    // Deserialize from json
    let json1 = '["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]'
    let msg = RelayMessage.fromJson(json1)

    // Serialize as json
    let json2 = msg.asJson()
    console.log(json2);
}

module.exports.relayMessageJson = relayMessageJson;
