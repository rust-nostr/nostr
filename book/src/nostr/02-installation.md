## Installing the library

=== "Rust"

    Add the `nostr` dependency in your `Cargo.toml` file:

    ```toml
    [dependencies]
    nostr = "0.24"
    ```

    Alternatively, you can add it directly from `git` source:

    ```toml
    [dependencies]
    nostr = { git = "https://github.com/rust-nostr/nostr", tag = "v0.24.0" }
    ```

    !!! note
        To use a specific commit, use `rev` instead of `tag`.

    Import the library in your code:

    ```rust
    use nostr::prelude::*;
    ```

=== "Python"

    The `nostr-protocol` package is available on the public PyPI:

    ```bash
    pip install nostr-protocol 
    ```

    Import the library in your code:

    ```python
    from nostr_protocol import *
    ```

=== "Kotlin"

    To use the Kotlin language bindings for `nostr` in your Android project add the following to your gradle dependencies:

    ```kotlin
    repositories {
        mavenCentral()
    }

    dependencies { 
        implementation("io.github.rust-nostr:nostr:<version>")
    }
    ```

    Import the library in your code:

    ```kotlin
    import nostr.*
    ```

    ## Known issues

    ### JNA dependency

    Depending on the JVM version you use, you might not have the JNA dependency on your classpath. The exception thrown will be

    ```bash
    class file for com.sun.jna.Pointer not found
    ```

    The solution is to add JNA as a dependency like so:

    ```kotlin
    dependencies {
        // ...
        implementation("net.java.dev.jna:jna:5.12.1")
    }
    ```

=== "Swift"

    ### Xcode

    Via `File > Add Packages...`, add

    ```
    https://github.com/rust-nostr/nostr-swift.git
    ```

    as a package dependency in Xcode.

    ### Swift Package

    Add the following to the dependencies array in your `Package.swift`:

    ``` swift
    .package(url: "https://github.com/rust-nostr/nostr-swift.git", from: "0.0.4"),
    ```

    Import the library in your code:

    ```swift
    import Nostr
    ```