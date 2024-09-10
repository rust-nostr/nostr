const keys = require("./src/keys");
const eventJson = require("./src/event/json");
const eventBuilder = require("./src/event/builder");
const relayMessageJson = require("./src/messages/relay");
const nip01 = require("./src/nip01");
const nip05 = require("./src/nip05");
const nip19 = require("./src/nip19");
const nip44 = require("./src/nip44");
const nip59 = require("./src/nip59");

async function main() {
    // Keys
    keys.generate();
    keys.restore();
    keys.vanity();

    eventJson.eventJson();
    eventBuilder.eventBuilder();

    relayMessageJson.relayMessageJson();

    nip01.run();
    await nip05.run();
    nip19.run();
    nip44.run();

    nip59.run();
}

main();
