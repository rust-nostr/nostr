const { PublicKey, loadWasmAsync, initLogger, LogLevel, NostrZapper, NostrWalletConnectURI, Client, ZapEntity } = require("../");
const {NWC} = require("../pkg/nostr_sdk_js");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    let uri = NostrWalletConnectURI.parse("nostr+walletconnect://..");

    let nwc = new NWC(uri);
    let zapper = NostrZapper.nwc(nwc);
    let client = Client.builder().zapper(zapper).build();

    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");
    await client.connect();

    let pk = PublicKey.fromBech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");
    let to = ZapEntity.publicKey(pk);
    await client.zap(to, 1000)
}

main();
