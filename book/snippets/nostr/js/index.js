const keys = require("./src/keys");
const eventJson = require("./src/event/json");
const eventBuilder = require("./src/event/builder");
const filters = require("./src/messages/filters");
const relayMessages = require("./src/messages/relay");
const nip01 = require("./src/nip01");
const nip05 = require("./src/nip05");
const nip06 = require("./src/nip06");
const nip19 = require("./src/nip19");
const nip21 = require("./src/nip21");
const nip44 = require("./src/nip44");
const nip59 = require("./src/nip59");
const nip65 = require("./src/nip65");

async function main() {
    // Keys
    keys.generate();
    keys.restore();
    keys.vanity();

    eventJson.eventJson();
    eventBuilder.eventBuilder();

    await relayMessages.run();
    await filters.run();

    nip01.run();
    await nip05.run();
    nip06.run();
    nip19.run();
    nip21.run();
    nip44.run();

    await nip59.run();
    nip65.run();
}

main();
