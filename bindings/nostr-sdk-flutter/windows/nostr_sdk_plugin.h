#ifndef FLUTTER_PLUGIN_NOSTR_SDK_PLUGIN_H_
#define FLUTTER_PLUGIN_NOSTR_SDK_PLUGIN_H_

#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>

#include <memory>

namespace nostr_sdk {

class NostrSdkPlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows *registrar);

  NostrSdkPlugin();

  virtual ~NostrSdkPlugin();

  // Disallow copy and assign.
  NostrSdkPlugin(const NostrSdkPlugin&) = delete;
  NostrSdkPlugin& operator=(const NostrSdkPlugin&) = delete;

  // Called when a method is called on this plugin's channel from Dart.
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &method_call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result);
};

}  // namespace nostr_sdk

#endif  // FLUTTER_PLUGIN_NOSTR_SDK_PLUGIN_H_
