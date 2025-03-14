#include <jni.h>
#include "nostr-sdk-rn.h"

extern "C"
JNIEXPORT jdouble JNICALL
Java_com_nostrsdkrn_NostrSdkRnModule_nativeMultiply(JNIEnv *env, jclass type, jdouble a, jdouble b) {
    return nostrsdkrn::multiply(a, b);
}
