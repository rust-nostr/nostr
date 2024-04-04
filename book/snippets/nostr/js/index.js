const keys = require("./src/keys");
const eventJson = require("./src/event/json");
const eventBuilder = require("./src/event/builder");
const relayMessageJson = require("./src/messages/relay");
const nip44 = require("./src/nip44");
const vanity = require("./src/vanity");

keys.keys();

eventJson.eventJson();

eventBuilder.eventBuilder();

relayMessageJson.relayMessageJson();

nip44.run();

vanity.vanity();