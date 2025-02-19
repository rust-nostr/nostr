// ANCHOR: full
import 'package:nostr_sdk/nostr_sdk.dart';

void encrypt() {
  // ANCHOR: parse-secret-key
  SecretKey secretKey = SecretKey.parse(secretKey: "3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683");
  // ANCHOR_END: parse-secret-key

  // ANCHOR: encrypt-default
  final password = "nostr";
  EncryptedSecretKey encrypted = secretKey.encrypt(password: password);
  // ANCHOR_END: encrypt-default

  print("Encrypted secret key: ${encrypted.toBech32()}");

  // ANCHOR: encrypt-custom
  EncryptedSecretKey encryptedCustom = EncryptedSecretKey(secretKey: secretKey, password: password, logN: 12, keySecurity: EncryptedSecretKeySecurity.weak);
  // ANCHOR_END: encrypt-custom

  print("Encrypted secret key (custom): ${encryptedCustom.toBech32()}");
}

void restore() {
  // ANCHOR: parse-ncryptsec
  EncryptedSecretKey encrypted = EncryptedSecretKey.fromBech32(bech32: "ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p");
  // ANCHOR_END: parse-ncryptsec

  // ANCHOR: decrypt
  SecretKey secretKey = encrypted.decrypt(password: "nostr");
  // ANCHOR_END: decrypt

  print("Decrypted secret key: ${secretKey.toBech32()}");
}
// ANCHOR_END: full
