package rust.nostr.snippets

// ANCHOR: full
import rust.nostr.sdk.*

suspend fun nip47() {
    // Parse NWC uri
    val uri = NostrWalletConnectUri.parse("nostr+walletconnect://..")

    // Initialize NWC client
    val nwc = Nwc(uri)

    // Get info
    val info = nwc.getInfo()
    println("Supported methods: ${info.methods}")

    // Get balance
    val balance = nwc.getBalance()
    println("Balance: $balance SAT")

    // Pay an invoice
    nwc.payInvoice("lnbc..")

    // Make an invoice
    val params = MakeInvoiceRequestParams(amount = 100u, description = null, descriptionHash = null, expiry = null)
    val result = nwc.makeInvoice(params)
    println("Invoice: ${result.invoice}")
}
// ANCHOR_END: full
