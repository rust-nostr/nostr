// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::sha256;
use secp256k1::schnorr::Signature;
use secp256k1::{KeyPair, Message, Secp256k1, XOnlyPublicKey};

/// This is an incomplete attempt to test I'm doing bip340 correctly
/// csv from https://github.com/bitcoin/bips/blob/master/bip-0340/test-vectors.csv
fn main() -> Result<(), Box<dyn Error>> {
    type Record = HashMap<String, String>;
    let mut rdr = csv::Reader::from_path("examples/bip340.csv")?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        let index = record.get("index").ok_or("Couldn't get index");
        println!("Testing index {}", index?);
        let secp = Secp256k1::new();

        let message_str = record.get("message").ok_or("Couldn't get message");
        let message_hash = sha256::Hash::from_hex(message_str?)?;
        let message = Message::from_slice(&message_hash)?;

        let signature = record.get("signature").ok_or("Couldn't get signature");
        let sig = Signature::from_str(signature?)?;

        let pubkey_record = record.get("public key").ok_or("Couldn't get public key");
        let pubkey = match XOnlyPublicKey::from_str(pubkey_record?) {
            Ok(pubkey) => pubkey,
            Err(e) => {
                eprintln!("Invalid public key: {}", e);
                continue;
            }
        };

        let _verify_result = secp.verify_schnorr(&sig, &message, &pubkey);
        let secretkey_record = record.get("secret key").ok_or("Couldn't get secret key");

        let _keypair = match KeyPair::from_seckey_str(&secp, secretkey_record?) {
            Ok(keypair) => keypair,
            Err(e) => {
                eprintln!("Invalid secret key: {}", e);
                continue;
            }
        };
    }

    Ok(())
}
