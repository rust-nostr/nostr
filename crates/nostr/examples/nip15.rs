// Copyright (c) 2023 ProTom
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let keys = Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;

    let shipping = ShippingMethod::new("123", 5.50).name("DHL");

    let stall = StallData::new("123", "my test stall", "USD")
        .description("this is a test stall")
        .shipping(vec![shipping.clone()]);

    let stall_event = EventBuilder::stall_data(stall).sign_with_keys(&keys)?;
    println!("{}", stall_event.as_json());

    let product = ProductData::new("1", "123", "my test product", "USD")
        .description("this is a test product")
        .price(5.50)
        .shipping(vec![shipping.get_shipping_cost()])
        .images(vec!["https://example.com/image.png".into()])
        .categories(vec!["test".into()]);

    let product_event = EventBuilder::product_data(product).sign_with_keys(&keys)?;
    println!("{}", product_event.as_json());

    Ok(())
}
