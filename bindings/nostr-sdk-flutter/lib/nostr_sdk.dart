// The original content is temporarily commented out to allow generating a self-contained demo - feel free to uncomment later.
library nostr_sdk;

export 'src/rust/frb_generated.dart' show NostrSdk;

export 'src/rust/api/protocol/event.dart';
export 'src/rust/api/protocol/event/tag.dart';
export 'src/rust/api/protocol/key.dart';
export 'src/rust/api/protocol/key/public_key.dart';
export 'src/rust/api/protocol/key/secret_key.dart';
export 'src/rust/api/client.dart';

// import 'nostr_sdk_platform_interface.dart';

// class NostrSdk {
//   Future<String?> getPlatformVersion() {
//     return NostrSdkPlatform.instance.getPlatformVersion();
//   }
// }
