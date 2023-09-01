# Embedded

## Running

To run the embedded test, first prepare your environment:

```shell
make init
```

Then:

```shell
make run
```

Output should be something like:

```text
heap size 262144

Restored keys from bech32:
- Secret Key (hex): 9571a568a42b9e05646a349c783159b906b498119390df9a5a02667155128028
- Public Key (hex): aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4
- Secret Key (bech32): nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99
- Public Key (bech32): npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy

Restore keys from mnemonic:
- Secret Key (hex): 06992419a8fe821dd8de03d4c300614e8feefb5ea936b76f89976dcace8aebee
- Public Key (hex): 648777b13344158549551f215ab2885d71af8861456eebea7102b1c729fc2de2
- Secret Key (bech32): nsec1q6vjgxdgl6ppmkx7q02vxqrpf687a7674ymtwmufjaku4n52a0hq9glmaf
- Public Key (bech32): npub1vjrh0vfngs2c2j24rus44v5gt4c6lzrpg4hwh6n3q2cuw20u9h3q4qf4pg

Random keys (using FakeRng):
- Secret Key (hex): 3939393939393939393939393939393939393939393939393939393939393939
- Public Key (hex): 1ff10be221c7b140505038042f5cc86530e9851a0e6c70ee16c18268768c2e02
- Secret Key (bech32): nsec18yunjwfe8yunjwfe8yunjwfe8yunjwfe8yunjwfe8yunjwfe8yusu2d2eh
- Public Key (bech32): npub1rlcshc3pc7c5q5zs8qzz7hxgv5cwnpg6pek8pmskcxpxsa5v9cpqqk7k0t
```

Note that this heap size is required because of the amount of stack used by libsecp256k1 when initializing a context.

## Cleanup

After sourcing `scripts/env.sh` and _before_ building again using another target
you'll want to unset `RUSTFLAGS` otherwise you'll get linker errors.

```shell
unset RUSTFLAGS
```
