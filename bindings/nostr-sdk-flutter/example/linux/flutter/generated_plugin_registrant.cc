//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <nostr_sdk/nostr_sdk_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) nostr_sdk_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "NostrSdkPlugin");
  nostr_sdk_plugin_register_with_registrar(nostr_sdk_registrar);
}
