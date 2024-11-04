#include <stdio.h>
#include <string.h>

#include <nostr_sdk.h>

int main() {
    // Generate keys
    const Keys* keys = keys_generate();
    printf("Keys generated.\n");

    const char* public_key = keys_public_key(keys);
    printf("Public key: %s\n", public_key);

    return 0;
}
