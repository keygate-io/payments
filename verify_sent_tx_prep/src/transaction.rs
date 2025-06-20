use std::fs;
use alloy_rlp::encode;
use hex;
use serde_json::{Value, json};

/// Extract both network RLP and EIP-2718 encodings from a consensus transaction
pub fn get_rlp_encodings(consensus_tx: &alloy::consensus::TxEnvelope) -> (Vec<u8>, Vec<u8>) {
    // Get network RLP encoding
    let network_rlp = alloy_rlp::encode(consensus_tx);
    
    // Extract EIP-2718 encoding by removing the outer RLP wrapper
    let mut network_slice = network_rlp.as_slice();
    let header = alloy_rlp::Header::decode(&mut network_slice).unwrap();
    let eip2718_bytes = network_slice[..header.payload_length].to_vec();
    
    (network_rlp, eip2718_bytes)
}

/// Write RLP bytes to file formatted for Noir
pub fn write_rlp_to_file(eip2718_bytes: &[u8], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Pad to 256 bytes (TX_RLP_MAX)
    let mut padded_bytes = vec![0u8; 256];
    let copy_len = std::cmp::min(eip2718_bytes.len(), 256);
    padded_bytes[..copy_len].copy_from_slice(&eip2718_bytes[..copy_len]);
    
    // Format as array for Noir
    let formatted = format!("[{}]", 
        padded_bytes.iter()
            .map(|b| format!("{}", b))
            .collect::<Vec<String>>()
            .join(", ")
    );
    
    fs::write(filename, formatted)?;
    println!("RLP encoding written to {}", filename);
    println!("Copy this into your Prover.toml: tx_rlp = {}", 
        padded_bytes.iter()
            .map(|b| format!("{}", b))
            .collect::<Vec<String>>()
            .join(", ")
            .chars()
            .take(100)
            .collect::<String>() + "..."
    );
    
    Ok(())
}

/// Process transaction to get RLP encodings (sync operation)
pub fn process_transaction_rlp(tx: &alloy::rpc::types::Transaction) -> Result<Value, String> {
    let tx_index = tx.transaction_index.unwrap_or_default();
    let nibbles = crate::utils::index_to_nibbles(tx_index);
    let rlp_encoded_index = encode(&tx_index);

    // Convert to consensus transaction for RLP encoding
    match tx.clone().try_into() {
        Ok(consensus_tx) => {
            let (network_rlp, eip2718_bytes) = get_rlp_encodings(&consensus_tx);
            
            // Calculate hash from the inner transaction
            let tx_hash = tx.inner.hash();
            
            Ok(json!({
                "success": true,
                "transaction": {
                    "hash": format!("0x{}", hex::encode(tx_hash)),
                    "index": tx_index,
                    "nibbles": nibbles,
                    "nibble_path": crate::utils::format_nibble_path(&nibbles),
                    "rlp_encoded_index": format!("0x{}", hex::encode(&rlp_encoded_index))
                },
                "rlp_encodings": {
                    "network_rlp_hex": format!("0x{}", hex::encode(&network_rlp)),
                    "eip2718_rlp_hex": format!("0x{}", hex::encode(&eip2718_bytes)),
                    "eip2718_rlp_bytes": eip2718_bytes
                }
            }))
        }
        Err(e) => {
            Err(format!("Failed to convert transaction for RLP encoding: {}", e))
        }
    }
}
