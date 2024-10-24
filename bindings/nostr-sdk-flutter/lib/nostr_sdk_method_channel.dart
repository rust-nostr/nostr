import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'nostr_sdk_platform_interface.dart';

/// An implementation of [NostrSdkPlatform] that uses method channels.
class MethodChannelNostrSdk extends NostrSdkPlatform {
  /// The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('nostr_sdk');

  @override
  Future<String?> getPlatformVersion() async {
    final version =
        await methodChannel.invokeMethod<String>('getPlatformVersion');
    return version;
  }
}
