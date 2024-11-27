// ANCHOR: full
import { NWC, NostrWalletConnectURI, PayInvoiceRequest, MakeInvoiceRequest } from "@rust-nostr/nostr-sdk";

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
    let payInvoiceParams = new PayInvoiceRequest();
    payInvoiceParams.invoice = "lnbc..";
    await nwc.payInvoice(payInvoiceParams);

    // Make an invoice
    let makeInvoiceParams = new MakeInvoiceRequest();
    makeInvoiceParams.amount = BigInt(100);
    const result = await nwc.makeInvoice(makeInvoiceParams)
    console.log("Invoice: " + result.invoice);

    // Drop client
    nwc.free();
}
// ANCHOR_END: full
