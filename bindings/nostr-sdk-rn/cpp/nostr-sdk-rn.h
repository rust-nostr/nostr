#ifndef NOSTRSDKRN_H
#define NOSTRSDKRN_H
// Generated by uniffi-bindgen-react-native
#include <cstdint>
#include <jsi/jsi.h>
#include <ReactCommon/CallInvoker.h>

namespace nostrsdkrn {
  using namespace facebook;

  uint8_t installRustCrate(jsi::Runtime &runtime, std::shared_ptr<react::CallInvoker> callInvoker);
  uint8_t cleanupRustCrate(jsi::Runtime &runtime);
}

#endif /* NOSTRSDKRN_H */