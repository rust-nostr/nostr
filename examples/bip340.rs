use std::{collections::HashMap, error::Error, str::FromStr};

use bitcoin_hashes::{hex::FromHex, sha256};
use secp256k1::{schnorrsig, Message, Secp256k1};

/// This is an incomplete attempt to test I'm doing bip340 correctly
/// csv from https://github.com/bitcoin/bips/blob/master/bip-0340/test-vectors.csv
fn main() -> Result<(), Box<dyn Error>> {
    type Record = HashMap<String, String>;
    let mut rdr = csv::Reader::from_path("bip340.csv").unwrap();
    for result in rdr.deserialize() {
        let record: Record = result.unwrap();
        println!("Testing index {}", record.get("index").unwrap());
        //let id = sha256::Hash::from_hex(id).unwrap();
        let secp = Secp256k1::new();

        let message_str = record.get("message").unwrap();
        let message_hash = sha256::Hash::from_hex(message_str).unwrap();

        let sig = schnorrsig::Signature::from_str(record.get("signature").unwrap()).unwrap();
        let message = Message::from(message_hash);

        let pubkey = match schnorrsig::PublicKey::from_str(record.get("public key").unwrap()) {
            Ok(pubkey) => pubkey,
            Err(e) => {
                eprintln!("Invalid public key: {}", e);
                continue;
            }
        };

        let _verify_result = secp.schnorrsig_verify(&sig, &message, &pubkey);

        let _keypair =
            match schnorrsig::KeyPair::from_seckey_str(&secp, record.get("secret key").unwrap()) {
                Ok(keypair) => keypair,
                Err(e) => {
                    eprintln!("Invalid secret key: {}", e);
                    continue;
                }
            };
    }

    Ok(())
}
