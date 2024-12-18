// ANCHOR: full

import 'package:nostr_sdk/nostr_sdk.dart';

Future<void> hello() async {
  // ANCHOR: client
  Keys keys = Keys.generate();
  NostrSigner signer = NostrSigner.keys(keys: keys);
  Client client = Client.builder().signer(signer: signer).build();
  // ANCHOR_END: client

  // ANCHOR: connect
  await client.addRelay(url: "wss://relay.damus.io");
  await client.connect();
  // ANCHOR_END: connect

  // ANCHOR: publish
  // TODO
  // ANCHOR_END: publish

  // ANCHOR: output
  // TODO
  // ANCHOR_END: output
}
// ANCHOR_END: full
