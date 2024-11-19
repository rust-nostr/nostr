// ANCHOR: full
import { NWC, NostrWalletConnectURI, MakeInvoiceRequestParams } from "@rust-nostr/nostr-sdk";

export async function main() {
    // Parse NWC uri
    let uri = NostrWalletConnectURI.parse("nostr+walletconnect://..");

    // Initialize NWC client
    let nwc = new NWC(uri);

    // Get info
    let info = await nwc.getInfo();
    console.log("Supported methods: ", info.methods);

    // Get balance
    let balance = await nwc.getBalance();
    console.log("Balance: " + balance + " SAT");

    // Pay an invoice
    await nwc.payInvoice("lnbc..")

    // Make an invoice
    let params = new MakeInvoiceRequestParams();
    params.amount = BigInt(100);
    const result = await nwc.makeInvoice(params)
    console.log("Invoice: " + result.invoice);

    // Drop client
    nwc.free();
}
// ANCHOR_END: full
