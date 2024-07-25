// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nostr_sdk::prelude::*;

// Check `nwc` crate for high level client library!

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut nwc_uri_string = String::new();
    let mut invoice = String::new();

    println!("Please enter a NWC string");
    std::io::stdin()
        .read_line(&mut nwc_uri_string)
        .expect("Failed to read line");

    println!("Please enter a BOLT 11 invoice");
    std::io::stdin()
        .read_line(&mut invoice)
        .expect("Failed to read line");

    invoice = String::from(invoice.trim());

    let nwc_uri =
        NostrWalletConnectURI::from_str(&nwc_uri_string).expect("Failed to parse NWC URI");

    let client = Client::default();
    client.add_relay(nwc_uri.relay_url.clone()).await?;

    client.connect().await;
    println!("Connected to relay {}", nwc_uri.relay_url);

    let req = nip47::Request::pay_invoice(PayInvoiceRequestParams {
        id: None,
        invoice,
        amount: None,
    });
    let req_event = req.to_event(&nwc_uri).unwrap();

    let subscription = Filter::new()
        .author(nwc_uri.public_key.clone())
        .kind(Kind::WalletConnectResponse)
        .event(req_event.id)
        .since(Timestamp::now());

    client.subscribe(vec![subscription], None).await?;

    client.send_event(req_event).await.unwrap();

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event { event, .. } = notification {
                let res = nip47::Response::from_event(&nwc_uri, &event)?;
                let PayInvoiceResponseResult { preimage } = res.to_pay_invoice()?;
                println!("Payment sent. Preimage: {preimage}");
            }
            Ok(true)
        })
        .await?;

    Ok(())
}
