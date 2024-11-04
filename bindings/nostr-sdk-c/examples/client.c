#include <stdio.h>
#include <string.h>

#include <nostr_sdk.h>

int main() {
    // Init logger
    enum LogLevel level = Debug;
    init_logger(level);

    // New client without signer
    const Client* client = client_without_signer();

    // Add relays
    client_add_relay(client, "wss://relay.damus.io");

    // Connect
    client_connect(client);

    // Send event
    const char* json = "{\"content\":\"Think about this.\\n\\nThe most powerful centralized institutions in the world have been replaced by a protocol that protects the individual. #bitcoin\\n\\nDo you doubt that we can replace everything else?\\n\\nBullish on the future of humanity\\nnostr:nevent1qqs9ljegkuk2m2ewfjlhxy054n6ld5dfngwzuep0ddhs64gc49q0nmqpzdmhxue69uhhyetvv9ukzcnvv5hx7un8qgsw3mfhnrr0l6ll5zzsrtpeufckv2lazc8k3ru5c3wkjtv8vlwngksrqsqqqqqpttgr27\",\"created_at\":1703184271,\"id\":\"38acf9b08d06859e49237688a9fd6558c448766f47457236c2331f93538992c6\",\"kind\":1,\"pubkey\":\"e8ed3798c6ffebffa08501ac39e271662bfd160f688f94c45d692d8767dd345a\",\"sig\":\"f76d5ecc8e7de688ac12b9d19edaacdcffb8f0c8fa2a44c00767363af3f04dbc069542ddc5d2f63c94cb5e6ce701589d538cf2db3b1f1211a96596fabb6ecafe\",\"tags\":[[\"e\",\"5fcb28b72cadab2e4cbf7311f4acf5f6d1a99a1c2e642f6b6f0d5518a940f9ec\",\"\",\"mention\"],[\"p\",\"e8ed3798c6ffebffa08501ac39e271662bfd160f688f94c45d692d8767dd345a\",\"\",\"mention\"],[\"t\",\"bitcoin\"],[\"t\",\"bitcoin\"]]}";
    client_send_event(client, json);

    return 0;
}
