use secp256k1::{
    schnorrsig,
    schnorrsig::{KeyPair, PublicKey},
    Secp256k1, SecretKey,
};

use std::str::FromStr;

/// Given a secret key return all the scep256k1 primitives
pub fn gen_keys(sk: &str) -> (KeyPair, PublicKey, SecretKey) {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_str(sk).unwrap();
    let key_pair = schnorrsig::KeyPair::from_secret_key(&secp, sk);
    let pk = schnorrsig::PublicKey::from_keypair(&secp, &key_pair);

    return (key_pair, pk, sk);
}
