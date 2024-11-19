// ANCHOR: full
use nostr_sdk::prelude::*;

pub async fn run() -> Result<()> {
    // Parse NWC uri
    let uri = NostrWalletConnectURI::parse("nostr+walletconnect://..")?;

    // Initialize NWC client
    let nwc = NWC::new(uri);

    // Get info
    let info = nwc.get_info().await?;
    println!("Supported methods: {:?}", info.methods);

    // Get balance
    let balance = nwc.get_balance().await?;
    println!("Balance: {balance} SAT");

    // Pay an invoice
    nwc.pay_invoice("lnbc..").await?;

    // Make an invoice
    let params = MakeInvoiceRequestParams {
        amount: 100,
        description: None,
        description_hash: None,
        expiry: None,
    };
    let result = nwc.make_invoice(params).await?;
    println!("Invoice: {}", result.invoice);

    Ok(())
}
// ANCHOR_END: full