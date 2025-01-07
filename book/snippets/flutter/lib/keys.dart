import 'package:nostr_sdk/nostr_sdk.dart';

// ANCHOR: generate
void generate() {
  final keys = Keys.generate();

  final publicKey = keys.publicKey();
  final secretKey = keys.secretKey();

  print("Public key (hex): ${publicKey.toHex()}");
  print("Secret key (hex): ${secretKey.toSecretHex()}");

  print("Public key (bech32): ${publicKey.toBech32()}");
  print("Secret key (bech32): ${secretKey.toBech32()}");
}
// ANCHOR_END: generate

// ANCHOR: restore
void restore() {
  // Parse keys directly from secret key
  var keys = Keys.parse(
    secretKey:
        "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99",
  );

  // Parse secret key and construct keys
  var secretKey = SecretKey.parse(
    secretKey:
        "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99",
  );
  keys = Keys(secretKey: secretKey);

  // Restore from hex
  secretKey = SecretKey.parse(
    secretKey:
        "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
  );
  keys = Keys(secretKey: secretKey);

  print(keys);
}
// ANCHOR_END: restore

// ANCHOR: vanity
void vanity() {
  // NOTE: NOT SUPPORTED YET!
}
// ANCHOR_END: vanity
