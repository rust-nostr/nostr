use nostr_sdk::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub async fn quickstart() -> Result<()> {
    // Generate new keys
    let my_keys: Keys = Keys::generate();

    let client = Client::new(&my_keys);
    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    client.add_relay("wss://relay.damus.io").await?;
    client
        .add_relay_with_opts(
            "wss://relay.nostr.info",
            RelayOptions::new().proxy(proxy).write(false),
        )
        .await?;
    client
        .add_relay_with_opts(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            RelayOptions::new().proxy(proxy),
        )
        .await?;

    // Connect to relays
    client.connect().await;

    // Set metadata
    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("yuki@getalby.com")
        .custom_field("custom_field", "my value");

    // Update metadata
    client.set_metadata(&metadata).await?;

    let filter = Filter::new().kind(Kind::Metadata);
    client.subscribe(vec![filter]).await;

    // Handle the notifications and exit when you receive the needed event
    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event { event, .. } = notification {
            if event.kind == Kind::Metadata {
                // handle the event
                break; // Exit
            }
        }
    }

    Ok(())
}
