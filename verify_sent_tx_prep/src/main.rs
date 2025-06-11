use std::io::{self, Write};
use std::str::FromStr;
use std::process::Command;
use std::env;
use alloy::eips::BlockNumberOrTag;
use alloy::{
    primitives::{B256, keccak256, Address}, 
    providers::{Provider, ProviderBuilder}
};
use hex;
use alloy_rlp::{encode};
use alloy_trie::{HashBuilder, Nibbles};
use serde_json::{Value, json};

const RPC_URL: &str = "https://mainnet.infura.io/v3/ba2572c3cedd43deaa43fd9e00261c33";
const DEFAULT_TX_HASH: &str = "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";

fn index_to_nibbles(index: u64) -> Vec<u8> {
    let mut nibbles = Vec::new();
    let mut num = index;
    
    // Convert to nibbles (4 bits each)
    while num > 0 {
        nibbles.push((num & 0xF) as u8);
        num >>= 4;
    }
    
    // Reverse to get correct order
    nibbles.reverse();
    if nibbles.is_empty() {
        nibbles.push(0);
    }
    nibbles
}

fn get_rlp_encodings(consensus_tx: &alloy::consensus::TxEnvelope) -> (Vec<u8>, Vec<u8>) {
    // Get network RLP encoding
    let network_rlp = alloy_rlp::encode(consensus_tx);
    
    // Extract EIP-2718 encoding by removing the outer RLP wrapper
    let mut network_slice = network_rlp.as_slice();
    let header = alloy_rlp::Header::decode(&mut network_slice).unwrap();
    let eip2718_bytes = network_slice[..header.payload_length].to_vec();
    
    (network_rlp, eip2718_bytes)
}

async fn get_proof_nodes(block_number: u64, tx_hash: &str) -> Result<Value, Box<dyn std::error::Error>> {
    // Execute the node script
    let output = Command::new("node")
        .arg("proof_nodes/index.js")
        .arg("proof")
        .arg(block_number.to_string())
        .arg(tx_hash)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Node script failed: {}", stderr).into());
    }
    
    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: Value = serde_json::from_str(&stdout)?;
    
    Ok(result)
}

async fn process_transaction(tx_hash_input: &str, expected_to: &str, expected_value: &str) -> Value {
    // Parse the input into B256
    let tx_hash = match B256::from_str(tx_hash_input) {
        Ok(hash) => hash,
        Err(_) => {
            return json!({
                "success": false,
                "error": "Invalid transaction hash format"
            });
        }
    };

    // Connect to provider
    let provider = match ProviderBuilder::new().connect(RPC_URL).await {
        Ok(p) => p,
        Err(e) => {
            return json!({
                "success": false,
                "error": format!("Failed to connect to RPC: {}", e)
            });
        }
    };
    
    // Fetch transaction details
    let tx = match provider.get_transaction_by_hash(tx_hash).await {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            return json!({
                "success": false,
                "error": "Transaction not found"
            });
        }
        Err(e) => {
            return json!({
                "success": false,
                "error": format!("Error fetching transaction: {}", e)
            });
        }
    };

    let tx_index = tx.transaction_index.unwrap_or_default();
    let nibbles = index_to_nibbles(tx_index);
    let rlp_encoded_index = encode(&tx_index);

    let mut result = json!({
        "success": true,
        "transaction": {
            "hash": format!("0x{}", hex::encode(tx_hash)),
            "index": tx_index,
            "nibbles": nibbles,
            "nibble_path": nibbles.iter()
                .map(|n| format!("{:x}", n))
                .collect::<Vec<String>>()
                .join(""),
            "rlp_encoded_index": format!("0x{}", hex::encode(&rlp_encoded_index))
        },
        "expected_to": expected_to,
        "expected_value": expected_value
    });

    // Get block number for proof and receipt
    if let Some(block_number) = tx.block_number {
        // Get transaction receipt
        if let Ok(Some(receipt)) = provider.get_transaction_receipt(tx_hash).await {
            result["receipt"] = json!({
                "block_number": block_number,
                "transaction_index": receipt.transaction_index.unwrap_or_default()
            });
        }
        
        // Generate proof nodes
        match get_proof_nodes(block_number, &format!("0x{}", hex::encode(tx_hash))).await {
            Ok(proof_result) => {
                result["proof"] = proof_result;
            }
            Err(e) => {
                result["proof"] = json!({
                    "success": false,
                    "error": format!("Error generating proof nodes: {}", e)
                });
            }
        }
    }
    
    // Get RLP encodings
    match tx.try_into() {
        Ok(consensus_tx) => {
            let (network_rlp, eip2718_bytes) = get_rlp_encodings(&consensus_tx);
            result["rlp_encodings"] = json!({
                "network_rlp": format!("0x{}", hex::encode(&network_rlp)),
                "eip2718_bytes": format!("0x{}", hex::encode(&eip2718_bytes))
            });
        }
        Err(e) => {
            result["rlp_encodings"] = json!({
                "error": format!("Failed to convert transaction for RLP encoding: {}", e)
            });
        }
    }

    result
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        let error_result = json!({
            "success": false,
            "error": "Usage: keygate <transaction_hash> <expected_to> <expected_value>"
        });
        println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
        std::process::exit(1);
    }
    
    let tx_hash = &args[1];
    let expected_to = &args[2];
    let expected_value = &args[3];
    
    let result = process_transaction(tx_hash, expected_to, expected_value).await;
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
