#include "include/nostr_sdk/nostr_sdk_plugin_c_api.h"

#include <flutter/plugin_registrar_windows.h>

#include "nostr_sdk_plugin.h"

void NostrSdkPluginCApiRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  nostr_sdk::NostrSdkPlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
