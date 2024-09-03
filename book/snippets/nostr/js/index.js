const keys = require("./src/keys");
const eventJson = require("./src/event/json");
const eventBuilder = require("./src/event/builder");
const relayMessageJson = require("./src/messages/relay");
const nip01 = require("./src/nip01");
const nip44 = require("./src/nip44");
const nip59 = require("./src/nip59");

// Keys
keys.generate();
keys.restore();
keys.vanity();

eventJson.eventJson();
eventBuilder.eventBuilder();

relayMessageJson.relayMessageJson();

nip01.run();
nip44.run();

nip59.run();