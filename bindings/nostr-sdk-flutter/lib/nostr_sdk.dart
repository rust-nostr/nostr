// The original content is temporarily commented out to allow generating a self-contained demo - feel free to uncomment later.

//
// import 'nostr_sdk_platform_interface.dart';
//
// class NostrSdk {
//   Future<String?> getPlatformVersion() {
//     return NostrSdkPlatform.instance.getPlatformVersion();
//   }
// }
//

library nostr_sdk;

export 'src/rust/api/simple.dart';
export 'src/rust/frb_generated.dart' show RustLib;
