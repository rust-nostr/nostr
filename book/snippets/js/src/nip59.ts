import {Keys, EventBuilder, UnwrappedGift, NostrSigner} from "@rust-nostr/nostr-sdk";

export async function run() {
    // Sender Keys
    const alice_keys = Keys.parse("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a");
    const alice_signer = NostrSigner.keys(alice_keys);

    // Receiver Keys
    const bob_keys = Keys.parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99");
    const bob_signer = NostrSigner.keys(bob_keys);

    // Compose rumor
    const rumor = EventBuilder.textNote("Test", []).build(alice_keys.publicKey)

    // Build gift wrap with sender keys
    const gw = await EventBuilder.giftWrap(alice_signer, bob_keys.publicKey, rumor)
    console.log("Gift Wrap: " + gw.asJson())

    // Extract rumor from gift wrap with receiver keys
    let unwrapped_gift = await UnwrappedGift.fromGiftWrap(bob_signer, gw);
    console.log("Sender: ", unwrapped_gift.sender.toBech32())
    console.log("Rumor: ", unwrapped_gift.rumor.asJson())
}
