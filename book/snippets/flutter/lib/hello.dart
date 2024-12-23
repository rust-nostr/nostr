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
  EventBuilder builder = EventBuilder.textNote(content: "Hello, rust-nostr!");
  SendEventOutput output = await client.sendEventBuilder(builder: builder);
  // ANCHOR_END: publish

  // ANCHOR: output
  print("Event ID: ${output.id}");
  print("Sent to: ${output.success}");
  print("Not sent to: ${output.failed}");
  // ANCHOR_END: output
}
// ANCHOR_END: full
