const keys = require("./src/keys");
const eventJson = require("./src/event/json");
const eventBuilder = require("./src/event/builder");
const relayMessageJson = require("./src/messages/relay");
const nip44 = require("./src/nip44");
const nip59 = require("./src/nip59");
const vanity = require("./src/vanity");

keys.keys();

eventJson.eventJson();

eventBuilder.eventBuilder();

relayMessageJson.relayMessageJson();

nip44.run();

nip59.run();

vanity.vanity();