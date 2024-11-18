use nostr_sdk::prelude::*;

pub fn nip01() -> Result<()> {
    let keys = Keys::generate();
    let content = r#"{
        "name": "w3irdrobot",
        "picture": "https://w3ird.tech/images/avatar.png",
        "nip05": "rob@w3ird.tech",
        "lud06": "",
        "username": "w3irdrobot",
        "display_name": "w3irdrobot",
        "displayName": "w3irdrobot",
        "banner": "https://w3ird.tech/images/banner.png",
        "website": "https://w3ird.tech",
        "about": "send nodes",
        "lud16": "w3irdrobot@vlt.ge"
      }"#;
    // ANCHOR: create-metadata
    let event = EventBuilder::new(Kind::Metadata, content, vec![]).sign_with_keys(&keys)?;
    let metadata = Metadata::from_json(&event.content)?;
    // ANCHOR_END: create-metadata
    println!("nostr address: {}", metadata.lud16.unwrap());

    // ANCHOR: create-event
    let metadata = Metadata::from_json(content)?;
    let event = EventBuilder::metadata(&metadata).sign_with_keys(&keys)?;
    // ANCHOR_END: create-event
    println!("event: {:?}", event);

    Ok(())
}
