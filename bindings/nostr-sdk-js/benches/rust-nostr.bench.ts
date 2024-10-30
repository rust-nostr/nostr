import {
    loadWasmAsync,
    loadWasmSync,
} from "@rust-nostr/nostr-sdk";

// @ts-ignore
Deno.bench("loadWasmSync", () => {
    loadWasmSync();
});

// @ts-ignore
Deno.bench("loadWasmAsync", async () => {
    await loadWasmAsync();
});
