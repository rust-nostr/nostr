import 'package:flutter_test/flutter_test.dart';
import 'package:nostr_sdk/nostr_sdk_platform_interface.dart';
import 'package:nostr_sdk/nostr_sdk_method_channel.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class MockNostrSdkPlatform
    with MockPlatformInterfaceMixin
    implements NostrSdkPlatform {
  @override
  Future<String?> getPlatformVersion() => Future.value('42');
}

void main() {
  final NostrSdkPlatform initialPlatform = NostrSdkPlatform.instance;

  test('$MethodChannelNostrSdk is the default instance', () {
    expect(initialPlatform, isInstanceOf<MethodChannelNostrSdk>());
  });

  test('getPlatformVersion', () async {
    NostrSdk nostrSdkPlugin = NostrSdk();
    MockNostrSdkPlatform fakePlatform = MockNostrSdkPlatform();
    NostrSdkPlatform.instance = fakePlatform;

    expect(await nostrSdkPlugin.getPlatformVersion(), '42');
  });
}
