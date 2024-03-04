const keys = require("./src/keys");
const eventJson = require("./src/event/json");
const eventBuilder = require("./src/event/builder");
const relayMessageJson = require("./src/messages/relay");

keys.keys();

eventJson.eventJson();

eventBuilder.eventBuilder();

relayMessageJson.relayMessageJson();