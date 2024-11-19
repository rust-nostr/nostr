import { PublicKey, Metadata, verifyNip05, getNip05Profile } from "@rust-nostr/nostr-sdk";

export async function run() {
    console.log();
    // ANCHOR: set-metadata
    // Create metadata object with name and NIP05
    let metadata = new Metadata()
        .name("TestName")
        .nip05("TestName@rustNostr.com");
    // ANCHOR_END: set-metadata

    console.log();
    // ANCHOR: verify-nip05
    console.log("Verify NIP-05:");
    let nip05 = "Rydal@gitlurker.info";
    let publicKey = PublicKey.parse("npub1zwnx29tj2lnem8wvjcx7avm8l4unswlz6zatk0vxzeu62uqagcash7fhrf");
    if (await verifyNip05(publicKey, nip05)) {
        console.log(`     '${nip05}' verified, for ${publicKey.toBech32()}`);
    } else {
        console.log(`     Unable to verify NIP-05, for ${publicKey.toBech32()}`);
    };
    // ANCHOR_END: verify-nip05

    console.log();

    // ANCHOR: nip05-profile
    console.log("Get NIP-05 profile:");
    let nip_05 = "yuki@yukikishimoto.com";
    let profile = await getNip05Profile(nip_05);
    console.log(`     ${nip_05} Public key: ${profile.publicKey().toBech32()}`);
    // ANCHOR_END: nip05-profile
}
