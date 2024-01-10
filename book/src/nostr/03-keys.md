# Keys

## Generate new random keys

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/nostr/rust/src/keys.rs}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/nostr/python/src/keys.py}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```javascript,ignore
{{#include ../../snippets/nostr/js/src/keys.js}}
```

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>
</custom-tabs>

## Restore from **hex** and/or **bech32** secret key

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
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

    Ok(())
}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
from nostr_protocol import *

secret_key = SecretKey.from_hex("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
keys = Keys(secret_key)

secret_key = SecretKey.from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
keys = Keys(secret_key)

keys = Keys.from_sk_str("hex or bech32 secret key")
```

</section>

<div slot="title">JavaScript</div>
<section>

TODO

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>
</custom-tabs>