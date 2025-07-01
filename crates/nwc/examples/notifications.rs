// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::env;
use std::str::FromStr;

use nostr::nips::nip47::{NotificationType, PaymentNotification};
use nwc::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,nwc=debug,nostr_relay_pool=debug".to_string());

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(true)
        .with_line_number(true)
        .init();

    let uri_str = env::args()
        .nth(1)
        .or_else(|| env::var("NWC_URI").ok())
        .ok_or("Please provide NWC URI as argument or set NWC_URI environment variable")?;

    println!("🔗 Connecting to wallet service...");

    let uri =
        NostrWalletConnectURI::from_str(&uri_str).map_err(|e| format!("Invalid NWC URI: {}", e))?;

    println!("📡 Relay: {:?}", uri.relays);
    println!("🔑 Wallet pubkey: {}", uri.public_key);

    let nwc = NWC::new(uri);

    nwc.subscribe_to_notifications().await?;

    println!("✅ NWC client started and listening for notifications...");
    println!("📱 Wallet notifications will appear here in real-time");
    println!("💡 Try making or receiving payments with your wallet");
    println!("🛑 Press Ctrl+C to stop");

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    tokio::select! {
        _ = shutdown => {
            println!("\n👋 Shutting down...");
        }

        result = nwc.handle_notifications(|notification| async move {
            match notification.notification_type {
                NotificationType::PaymentReceived => {
                    if let Ok(payment) = notification.to_pay_notification() {
                        println!("🟢 Payment Received!");
                        print_payment_details(&payment);
                    }
                }
                NotificationType::PaymentSent => {
                    if let Ok(payment) = notification.to_pay_notification() {
                        println!("🔴 Payment Sent!");
                        print_payment_details(&payment);
                    }
                }
            }
            Ok::<bool, Box<dyn std::error::Error>>(false) // Continue processing
        }) => {
            if let Err(e) = result {
                eprintln!("Error handling notifications: {}", e);
            }
        }
    }

    nwc.unsubscribe_from_notifications().await?;

    nwc.shutdown().await;

    Ok(())
}

fn print_payment_details(payment: &PaymentNotification) {
    println!("  💰 Amount: {} msat", payment.amount);
    if let Some(description) = &payment.description {
        println!("  📝 Description: {}", description);
    }
    println!("  🔗 Payment Hash: {}", payment.payment_hash);
    println!("  📅 Settled At: {}", payment.settled_at);
    if payment.fees_paid > 0 {
        println!("  💸 Fees: {} msat", payment.fees_paid);
    }
    println!();
}
