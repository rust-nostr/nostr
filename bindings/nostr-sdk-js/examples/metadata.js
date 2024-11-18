const { Metadata, loadWasmSync } = require("../");

function main() {
    loadWasmSync();
    
    let metadata = new Metadata().name("test").displayName("Testing Rust Nostr").lud16("pay@yukikishimoto.com");

    console.log("JSON:", metadata.asJson());
    console.log("Display name:", metadata.getDisplayName());
}

main();
