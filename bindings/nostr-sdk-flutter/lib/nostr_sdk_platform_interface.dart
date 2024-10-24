import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'nostr_sdk_method_channel.dart';

abstract class NostrSdkPlatform extends PlatformInterface {
  /// Constructs a NostrSdkPlatform.
  NostrSdkPlatform() : super(token: _token);

  static final Object _token = Object();

  static NostrSdkPlatform _instance = MethodChannelNostrSdk();

  /// The default instance of [NostrSdkPlatform] to use.
  ///
  /// Defaults to [MethodChannelNostrSdk].
  static NostrSdkPlatform get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [NostrSdkPlatform] when
  /// they register themselves.
  static set instance(NostrSdkPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<String?> getPlatformVersion() {
    throw UnimplementedError('platformVersion() has not been implemented.');
  }
}
