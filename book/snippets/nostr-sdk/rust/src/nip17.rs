use nostr_sdk::prelude::*;
use nostr_relay_builder::prelude::*;

pub async fn run() -> Result<()> {
    let relay = MockRelay::run().await?;
    let url = relay.url();

    // ANCHOR: nip17
    // Sender
    let alice_keys =
        Keys::parse("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")?;
    let alice_client = Client::new(alice_keys);
    alice_client.add_relay(url.clone()).await?;
    alice_client.connect().await;

    // Receiver
    let bob_keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let bob_client = Client::new(bob_keys.clone());
    bob_client.add_relay(url.clone()).await?;
    bob_client.connect().await;

    // Initialize subscription on Bob's side before sending the message
    // Limit is set to 0 to get only new events!
    // Timestamp::now() CAN'T be used for gift wrap since the timestamps are tweaked!
    let message_filter = Filter::new()
        .kind(Kind::GiftWrap)
        .pubkey(bob_keys.public_key())
        .limit(0);
    let subscription_id = bob_client.subscribe(vec![message_filter], None).await?;

    // Alice sends private message to Bob
    alice_client.send_private_msg(bob_keys.public_key(), "Hello Bob!".to_string(), None).await?;
    println!("Sent private message to Bob");

    // Bob receives private message
    bob_client.handle_notifications(|notification| async {
        if let RelayPoolNotification::Event { event, .. } = notification {
            if event.kind == Kind::GiftWrap {
                let UnwrappedGift { rumor, sender } = bob_client.unwrap_gift_wrap(&event).await?;
                if rumor.kind == Kind::PrivateDirectMessage {
                    println!("Bob received private message from {sender}: {}", &rumor.content);
                    return Ok(true); // Message received, exit.
                }
            }
        }
        Ok(false)
    }).await?;

    bob_client.unsubscribe(subscription_id.val).await;

    // ANCHOR_END: nip17

    Ok(())
}
