# Keys

## Generate new random keys

=== "Rust"

    ```rust
    use nostr::prelude::*;

    fn main() -> Result<()> {
        let keys = Keys::generate();
        
        let public_key = keys.public_key();
        let secret_key = keys.secret_key()?;

        println!("Public key (hex): {}", public_key);
        println!("Public key (bech32): {}", public_key.to_bech32()?);
        println!("Secret key (hex): {}", keys.secret_key()?.display_secret());
        println!("Secret key (bech32): {}", secret_key.to_bech32()?);
    }
    ```

=== "Python"

    ```python
    from nostr_protocol import Keys

    keys = Keys.generate()
    public_key = keys.public_key()
    secret_key = keys.secret_key()
    print("Keys:")
    print(" Public keys:")
    print(f"     hex:    {public_key.to_hex()}")
    print(f"     bech32: {public_key.to_bech32()}")
    print(" Secret keys:")
    print(f"     hex:    {secret_key.to_hex()}")
    print(f"     bech32: {secret_key.to_bech32()}")
    ```

=== "Kotlin"

    ```kotlin
    TODO
    ```

=== "Swift"

    ``` swift
    TODO
    ```

## Restore from **hex** and/or **bech32** secret key

=== "Rust"

    ```rust
    use std::str::FromStr;

    use nostr::prelude::*;

    fn main() -> Result<()> {
        // Restore from hex
        let secret_key = SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;
        let keys = Keys::new(secret_key);

        // Restore from bech32
        let secret_key = SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
        let keys = Keys::new(secret_key);

        // Try from bech32 or hex
        let keys = Keys::from_sk_str("hex or bech32 secret key")?;
    
        // ...
    }
    ```

=== "Python"

    ```python
    from nostr_protocol import *

    secret_key = SecretKey.from_hex("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
    keys = Keys(secret_key)

    secret_key = SecretKey.from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
    keys = Keys(secret_key)

    keys = Keys.from_sk_str("hex or bech32 secret key")

    # ..
    ```

=== "Kotlin"

    ```kotlin
    TODO
    ```

=== "Swift"

    ``` swift
    TODO
    ```