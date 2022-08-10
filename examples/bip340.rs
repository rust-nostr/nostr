use std::{collections::HashMap, error::Error, str::FromStr};

use bitcoin_hashes::{hex::FromHex, sha256};
use secp256k1::{schnorrsig, Message, Secp256k1};

/// This is an incomplete attempt to test I'm doing bip340 correctly
/// csv from https://github.com/bitcoin/bips/blob/master/bip-0340/test-vectors.csv
fn main() -> Result<(), Box<dyn Error>> {
    type Record = HashMap<String, String>;
    let mut rdr = csv::Reader::from_path("bip340.csv")?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        let index = record.get("index").ok_or("Couldn't get index");
        println!("Testing index {}", index?);
        let secp = Secp256k1::new();

        let message_str = record.get("message").ok_or("Couldn't get message");
        let message_hash = sha256::Hash::from_hex(message_str?)?;
        let message = Message::from_slice(&message_hash)?;

        let signature = record.get("signature").ok_or("Couldn't get signature");
        let sig = schnorrsig::Signature::from_str(signature?)?;

        let pubkey_record = record.get("public key").ok_or("Couldn't get public key");
        let pubkey = match schnorrsig::PublicKey::from_str(pubkey_record?) {
            Ok(pubkey) => pubkey,
            Err(e) => {
                eprintln!("Invalid public key: {}", e);
                continue;
            }
        };

        let _verify_result = secp.schnorrsig_verify(&sig, &message, &pubkey);
        let secretkey_record = record.get("secret key").ok_or("Couldn't get secret key");

        let _keypair = match schnorrsig::KeyPair::from_seckey_str(&secp, secretkey_record?) {
            Ok(keypair) => keypair,
            Err(e) => {
                eprintln!("Invalid secret key: {}", e);
                continue;
            }
        };
    }

    Ok(())
}
