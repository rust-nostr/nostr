import 'package:flutter/material.dart';
import 'package:nostr_sdk/nostr_sdk.dart';

void main() async {
  await NostrSdk.init();
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  @override
  Widget build(BuildContext context) {
    final keys = Keys.generate();
    final publicKeyHex = keys.publicKey().toHex();
    print(publicKeyHex);

    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          title: const Text('Nostr Sdk Example'),
        ),
        body: Center(
            child: Column(
          children: [Text('pubkey hex: $publicKeyHex')],
        )),
      ),
    );
  }
}
