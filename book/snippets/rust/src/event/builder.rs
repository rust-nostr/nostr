// ANCHOR: full
use nostr_sdk::prelude::*;

async fn sign_and_print<T>(signer: &T, builder: EventBuilder) -> Result<()> 
where
    T: NostrSigner,
{
    // ANCHOR: sign
    let event: Event = builder.sign(signer).await?;
    // ANCHOR_END: sign
    
    println!("{}", event.as_json());
    
    Ok(())
}

pub async fn event() -> Result<()> {
    let keys = Keys::generate();

    // ANCHOR: standard
    let builder = EventBuilder::text_note("Hello");
    // ANCHOR_END: standard
    
    sign_and_print(&keys, builder).await?;

    // ANCHOR: std-custom
    let builder =
        EventBuilder::text_note("Hello with POW")
            .tag(Tag::alt("POW text-note"))
            .pow(20)
            .custom_created_at(Timestamp::from_secs(1737976769));
    // ANCHOR_END: std-custom

    sign_and_print(&keys, builder).await?;

    // ANCHOR: custom
    let builder = EventBuilder::new(Kind::Custom(33001), "My custom event");
    // ANCHOR_END: custom

    sign_and_print(&keys, builder).await
}
// ANCHOR_END: full
