use std::io::{self, Write};
use std::str::FromStr;
use alloy::eips::BlockNumberOrTag;
use alloy::{
    primitives::{B256, keccak256, Address}, 
    providers::{Provider, ProviderBuilder}
};
use hex;
use alloy_rlp::{encode};
use alloy_trie::{HashBuilder, Nibbles};
use std::process::Command;
use serde_json::Value;

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

async fn get_proof_nodes_from_js(block_number: u64, tx_hash: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new("node")
        .arg("proof_nodes/index.js")
        .arg("proof")
        .arg(block_number.to_string())
        .arg(tx_hash)
        .output()?;

    if !output.status.success() {
        return Err(format!("Node.js script failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let json_output = String::from_utf8_lossy(&output.stdout);
    let proof_data: Value = serde_json::from_str(&json_output)?;
    
    Ok(proof_data)
}

#[tokio::main]
async fn main() {
    let provider = ProviderBuilder::new().connect(RPC_URL).await.unwrap();
    
    print!("Enter transaction hash (press Enter for default): ");
    io::stdout().flush().unwrap();
    
    let mut tx_hash_input = String::new();
    io::stdin()
        .read_line(&mut tx_hash_input)
        .expect("Failed to read transaction hash");

    // Remove trailing whitespace
    let tx_hash_input = tx_hash_input.trim();
    
    // Use default hash if input is empty
    let tx_hash_input = if tx_hash_input.is_empty() {
        println!("Using default transaction hash: {}", DEFAULT_TX_HASH);
        DEFAULT_TX_HASH
    } else {
        tx_hash_input
    };
    
    // Parse the input into B256
    let tx_hash = match B256::from_str(tx_hash_input) {
        Ok(hash) => hash,
        Err(_) => {
            println!("Invalid transaction hash format");
            return;
        }
    };
    
    println!("Original Transaction Hash: 0x{}", hex::encode(tx_hash));
    
    // Fetch transaction details
    match provider.get_transaction_by_hash(tx_hash).await {
        Ok(Some(tx)) => {
            println!("\nTransaction found!");
            let tx_index = tx.transaction_index.unwrap_or_default();
            println!("Transaction Index: {}", tx_index);
            
            // Show nibble representation
            let nibbles = index_to_nibbles(tx_index);
            println!("Transaction Index as Nibbles: {:?}", nibbles);
            println!("Nibble Path in Trie: {}", nibbles.iter()
                .map(|n| format!("{:x}", n))
                .collect::<Vec<String>>()
                .join(""));
            
            // RLP encode the index
            let rlp_encoded_index = encode(&tx_index);
            println!("RLP Encoded Index: 0x{}", hex::encode(&rlp_encoded_index));

            // Get the block number for the proof
            if let Some(block_number) = tx.block_number {
                // Get the transaction receipt to verify inclusion
                if let Ok(Some(receipt)) = provider.get_transaction_receipt(tx_hash).await {
                    println!("\nTransaction Receipt:");
                    println!("Block Number: {}", block_number);
                    println!("Transaction Index: {}", receipt.transaction_index.unwrap_or_default());
                }
                
                // Generate the trie proof using Node.js script
                println!("\nGenerating proof nodes using Node.js script...");
                match get_proof_nodes_from_js(block_number, &format!("0x{}", hex::encode(tx_hash))).await {
                    Ok(proof_data) => {
                        println!("\nProof Nodes from Node.js:");
                        println!("{}", serde_json::to_string_pretty(&proof_data).unwrap());
                    }
                    Err(e) => println!("Error getting proof nodes from Node.js: {}", e),
                }
            }
            
            // Get RLP encoding via consensus conversion
            let consensus_tx: alloy::consensus::TxEnvelope = tx.try_into().unwrap();
            
            // Get both RLP encodings
            let (network_rlp, eip2718_bytes) = get_rlp_encodings(&consensus_tx);

            println!("Retrieved EIP-2718 RLP encoding: 0x{}", hex::encode(&eip2718_bytes));
        }
        Ok(None) => println!("Transaction not found"),
        Err(e) => println!("Error fetching transaction: {}", e),
    }
}
