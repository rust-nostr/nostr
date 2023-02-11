const { Keys, Client, Options, initLogger } = require("../index");

async function main() {
    initLogger();

    let keys = Keys.fromSkStr("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");
    let opts = new Options().waitForConnection(true).waitForSend(true);

    let client = Client.newWithOpts(keys, opts);
    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nostr.oxtr.dev");
    await client.addRelay("wss://nostr.bitcoiner.social");
    await client.addRelay("wss://nostr.openchain.fr");
    await client.addRelay("ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion", "127.0.0.1:9050");

    await client.connect();

    await client.publishTextNote("Hello World!", []);
}

main();