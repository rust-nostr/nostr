## Installing the library

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

Add the `nostr` dependency in your `Cargo.toml` file:

```toml,ignore
[dependencies]
nostr = "0.31"
```

Alternatively, you can add it directly from `git` source:

```toml,ignore
[dependencies]
nostr = { git = "https://github.com/rust-nostr/nostr", tag = "v0.31.0" }
```

```admonish info
To use a specific commit, use `rev` instead of `tag`.
```

Import the library in your code:

```rust,ignore
use nostr::prelude::*;
```

</section>

<div slot="title">Python</div>
<section>

The `nostr-protocol` package is available on the public PyPI:

```bash
pip install nostr-protocol 
```

Alternatively, you can manually add the dependency in your `requrements.txt`, `setup.py`, etc.:

```
nostr-protocol==0.12.1
```

Import the library in your code:

```python,ignore
from nostr_protocol import *
```

### Support matrix

The wheels are distributed for the following python `versions` and `platforms`.
If your `version`/`platform` is not currently supported, you can compile the wheel by your self following [these instructions](https://github.com/rust-nostr/nostr/blob/master/bindings/nostr-ffi/README.md#python).

#### Python version

| 3.8 | 3.9 | 3.10 | 3.11 | 3.12 | 3.13 |
| --- | --- | ---- | ---- | ---- | ---- |
| ❌  | ✅  |  ✅  |  ✅  |  ✅  |  ❌  |

#### Platform support

|   OS       | i686 | x64 | aarch64 | arm |
| ---------- | ---- | --- | ------- | --- |
| Linux      | ❌   | ✅  | ✅      | ❌  |
| macOS      | ❌   | ✅  | ✅      | ❌  |
| Windows    | ❌   | ✅  | ❌      | ❌  |

</section>

<div slot="title">JavaScript</div>
<section>

The `nostr` package is available on the public [npmjs](https://npmjs.com):

```bash
npm i @rust-nostr/nostr 
```

Alternatively, you can manually add the dependency in your `package.json` file:

```json
{
    "dependencies": {
        "@rust-nostr/nostr": "0.14.1"
    }
}
```

### WASM

This library to work **require** to load the WASM code.

#### Load in **async** context

```javascript,ignore
const { loadWasmAsync } = require("@rust-nostr/nostr");

async function main() {
    // Load WASM
    await loadWasmAsync();

    // ...
}

main();
```

#### Load in **sync** context

```javascript,ignore
const { loadWasmSync } = require("@rust-nostr/nostr");

function main() {
    // Load WASM
    loadWasmSync();

    // ...
}

main();
```

</section>

<div slot="title">Kotlin</div>
<section>

To use the Kotlin language bindings for `nostr` in your Android project add the following to your gradle dependencies:

```kotlin,ignore
repositories {
    mavenCentral()
}

dependencies { 
    implementation("io.github.rust-nostr:nostr:0.12.1")
}
```

Import the library in your code:

```kotlin
import rust.nostr.protocol.*
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
    implementation("net.java.dev.jna:jna:5.12.0@aar")
}
```

</section>

<div slot="title">Swift</div>
<section>

### Xcode

Via `File > Add Packages...`, add

```
https://github.com/rust-nostr/nostr-swift.git
```

as a package dependency in Xcode.

### Swift Package

Add the following to the dependencies array in your `Package.swift`:

``` swift
.package(url: "https://github.com/rust-nostr/nostr-swift.git", from: "0.12.1"),
```

Import the library in your code:

```swift
import Nostr
```

</section>
</custom-tabs>