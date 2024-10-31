#include <iostream>
#include <string>
#include <memory>

#include <nostr_sdk.h>

int main() {
    try {
        // Call the Rust functions from C++
        auto keys = key::generate();
        auto publicKey = keys->public_key();
        std::cout << "Generated Public Key: " << publicKey << std::endl;

        auto parsedKeys = key::parse("your_secret_key_here");
        auto parsedPublicKey = parsedKeys->public_key();
        std::cout << "Parsed Public Key: " << parsedPublicKey << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
    return 0;
}
