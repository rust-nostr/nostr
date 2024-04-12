const { NWC, NostrWalletConnectURI, loadWasmAsync, initLogger, LogLevel } = require("../");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    // Parse NWC uri
    let uri = NostrWalletConnectURI.parse("nostr+walletconnect://..");

    // Initialize NWC client
    let nwc = await new NWC(uri);

    // Get info
    let info = await nwc.getInfo();
    console.log("Supported methods: ", info.methods);

    // Get balance
    let balance = await nwc.getBalance();
    console.log("Balance: " + balance + " SAT");

    // Drop client
    nwc.free();
}

main();