package rust.nostr

import rust.nostr.sdk.Keys
import rust.nostr.sdk.SecretKey
import kotlin.test.Test

class TestKeys {
    @Test
    fun testKeys() {
        val keys: Keys = Keys.generate()

        val secretKey: SecretKey = keys.secretKey()

        // Serialize secret key to hex
        val hex: String = secretKey.toHex()

        // Parse hex
        val parsedKeys = Keys.parse(hex)

        assert(keys == parsedKeys) {
            "Keys doesn't match"
        }
    }
}
