# C/C++ bindings for Nostr SDK

## Integration

### Clone the repository

```bash
git clone https://github.com/rust-nostr/nostr.git
cd nostr
```

### Configure your CMake

```cmake
# Add the external library project
add_subdirectory(<path>/nostr/bindings/nostr-sdk-c nostr_sdk)
include_directories("<path>/nostr/bindings/nostr-sdk-c/include")
link_directories("<path>/nostr/target/release")

# Link library to a binary
target_link_libraries(<binary> PRIVATE nostr_sdk_interface)
```

### Build your project

```bash
mkdir build
cd build
cmake ..
make
```

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## Book

Learn more about `rust-nostr` at <https://rust-nostr.org>.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details
