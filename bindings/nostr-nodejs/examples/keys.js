const { Keys, SecretKey, PublicKey } = require("../index");

function main() {
    // Generate new random keys
    let keys = Keys.generate();
    console.log("Public key (hex): ", keys.publicKey().toHex());
    console.log("Secret key (hex): ", keys.secretKey().toHex());

    console.log("Public key (bech32): ", keys.publicKey().toBech32());
    console.log("Secret key (bech32): ", keys.secretKey().toBech32());

    let secretKey = SecretKey.fromBech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");
    let keys2 = new Keys(secretKey);
    console.log("Secret key (hex): ", keys2.secretKey().toHex());

    // Try to init Keys from hex or bech32 secret key
    let keys3 = Keys.fromSkStr("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");

    let publicKey = PublicKey.fromHex("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e");
    let keys4 = Keys.fromPublicKey(publicKey);
}

main();