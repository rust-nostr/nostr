use nostr::prelude::*;

pub fn run() -> Result<()> {
    // Sender keys
    let alice_keys =
        Keys::parse("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")?;
    let alice_client = Client::new(&keys);
    alice_client.add_relay("wss://relay.damus.io").await?;
    alice_client.connect().await;

    // Receiver Keys
    let bob_keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    bob_client.add_relay("wss://relay.damus.io").await?;
    bob_client.connect().await;
    // Initialize subscription on Bob's side before sending the message
    let subscription_id = init_subscription(&bob_keys, &bob_client).await?;

    // Alice sends private message to Bob
    alice_client.send_private_message(bob_keys.public_key(), "Hello Bob!".to_string(), None).await?;
    println!("Sent private message to Bob");

    // Bob receives private message
    match bob_client.notifications().recv().await {
        Ok(notification) => {
            if let RelayPoolNotification::Event { event, .. } = notification {
                let rumor = bob_client.unwrap_gift_wrap(&event).await?.rumor;
                if rumor.kind == Kind::PrivateDirectMessage {
                    println!("Bob received private message: {}", &rumor.content);
                }
            }
        }
        Err(err) => {
            println!("Bob got error: {}", err.to_string());
        }
    }

    bob_client.unsubscribe(subscription_id).await;

    Ok(())
}

async fn init_subscription(
    keys: &Keys,
    client: &Client,
) -> Result<SubscriptionId, Error> {
    let message_filter = Filter::new()
        .kind(Kind::GiftWrap)
        .pubkey(keys.public_key())
        .limit(0); // Limit set to 0 to get only new events! Timestamp::now() CAN'T be used for gift wrap since the timestamps are tweaked!
    let subscription_id = client.subscribe(vec![message_filter], None).await?.val;
    Ok(subscription_id)
}