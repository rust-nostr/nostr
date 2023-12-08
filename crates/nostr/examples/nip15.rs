// Copyright (c) 2023 ProTom
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

fn main() -> Result<()> {
    let alice_keys = Keys::from_sk_str(ALICE_SK)?;
    let shipping = ShippingMethod::new("123", 5.50).name("DHL");

    let stall = StallData::new("123", "my test stall", "USD")
        .description("this is a test stall")
        .shipping(vec![shipping.clone()]);

    let stall_event = EventBuilder::new_stall_data(stall).to_event(&alice_keys)?;
    println!("{}", stall_event.as_json());

    let product = ProductData::new("1", "123", "my test product", "USD")
        .description("this is a test product")
        .price(5.50)
        .shipping(vec![shipping.get_shipping_cost()])
        .images(vec!["https://example.com/image.png".into()])
        .categories(vec!["test".into()]);

    let product_event = EventBuilder::new_product_data(product).to_event(&alice_keys)?;
    println!("{}", product_event.as_json());

    Ok(())
}
