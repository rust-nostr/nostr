import 'package:flutter/material.dart';
import 'package:nostr_sdk/nostr_sdk.dart';

Future<void> main() async {
  await NostrSdk.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    final keys = Keys.generate();
    final publicKeyHex = keys.publicKey().toHex();
    print(publicKeyHex);

    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('Nostr SDK example')),
        body: Center(
          child: Text('pubkey hex: $publicKeyHex'),
        ),
      ),
    );
  }
}
