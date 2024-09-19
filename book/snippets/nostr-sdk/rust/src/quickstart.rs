use nostr_sdk::prelude::*;

pub async fn quickstart() -> Result<()> {
    // ANCHOR: create-client
    let my_keys: Keys = Keys::generate();

    let client = Client::new(&my_keys);

    client.add_relay("wss://relay.damus.io").await?;
    client.add_read_relay("wss://relay.nostr.info").await?;

    client.connect().await;
    // ANCHOR_END: create-client

    // ANCHOR: create-metadata
    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("yuki@getalby.com")
        .custom_field("custom_field", "my value");

    client.set_metadata(&metadata).await?;
    // ANCHOR_END: create-metadata

    // ANCHOR: create-filter
    let filter = Filter::new().kind(Kind::Metadata);
    let sub_id = client.subscribe(vec![filter], None).await?;
    // ANCHOR_END: create-filter

    // ANCHOR: notifications
    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event {
            subscription_id,
            event,
            ..
        } = notification
        {
            if subscription_id == *sub_id && event.kind == Kind::Metadata {
                // handle the event
                break; // Exit
            }
        }
    }
    // ANCHOR_END: notifications

    Ok(())
}
