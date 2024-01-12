use nostr::prelude::*;

pub fn nip_utilities() -> Result<()> {
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
    let event = EventBuilder::new(Kind::Metadata, content, vec![]).to_event(&keys)?;
    let metadata = Metadata::from_json(event.content)?;
    println!("nostr address: {}", metadata.lud16.unwrap());

    let metadata = Metadata::from_json(content)?;
    let event = EventBuilder::set_metadata(&metadata).to_event(&keys)?;
    println!("event: {:?}", event);

    Ok(())
}
