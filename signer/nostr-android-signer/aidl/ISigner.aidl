package com.nostr.signer;

interface ISigner {
    String getPublicKey();

    String signEvent(in String event);
}
