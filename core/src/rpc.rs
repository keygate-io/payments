use std::str::FromStr;
use alloy::{
    primitives::B256,
    providers::{Provider, ProviderBuilder}
};
use serde_json::{Value, json};
use hex;

/// Connect to the RPC provider
pub async fn connect_provider(rpc_url: &str) -> Result<impl Provider, Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().connect(rpc_url).await?;
    Ok(provider)
}

/// Fetch transaction details from RPC
pub async fn fetch_transaction_details(rpc_url: &str, tx_hash_input: &str) -> Result<alloy::rpc::types::Transaction, String> {
    // Parse the input into B256
    let tx_hash = B256::from_str(tx_hash_input)
        .map_err(|_| format!("Invalid transaction hash format. Received: {}", tx_hash_input))?;

    // Connect to provider
    let provider = connect_provider(rpc_url).await
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;
    
    // Fetch transaction details
    match provider.get_transaction_by_hash(tx_hash).await {
        Ok(Some(tx)) => Ok(tx),
        Ok(None) => Err("Transaction not found".to_string()),
        Err(e) => Err(format!("Error fetching transaction: {}", e)),
    }
}

/// Full transaction processing with RPC, proof generation, and RLP encoding
pub async fn process_full_transaction(
    rpc_url: &str,
    tx_hash_input: &str, 
    expected_to: &str, 
    expected_value: &str, 
) -> Value {
    // Fetch transaction details
    let tx = match fetch_transaction_details(rpc_url, tx_hash_input).await {
        Ok(tx) => tx,
        Err(error) => {
            return json!({
                "success": false,
                "error": error
            });
        }
    };

    // Get basic transaction info and RLP encodings (sync operation)
    let mut result = match crate::transaction::process_transaction_rlp(&tx) {
        Ok(mut result) => {
            // Add expected values
            result["expected_to"] = json!(expected_to);
            result["expected_value"] = json!(expected_value);
            result
        }
        Err(error) => {
            return json!({
                "success": false,
                "error": error
            });
        }
    };

    // Get block number for proof and receipt
    if let Some(block_number) = tx.block_number {
        // Get transaction receipt
        if let Ok(provider) = connect_provider(rpc_url).await {
            let tx_hash = B256::from_str(tx_hash_input).unwrap();
            if let Ok(Some(receipt)) = provider.get_transaction_receipt(tx_hash).await {
                result["receipt"] = json!({
                    "block_number": block_number,
                    "transaction_index": receipt.transaction_index.unwrap_or_default()
                });
            }
        }
        
        // Generate proof nodes
        match crate::proof::get_proof_nodes(rpc_url, block_number, &format!("0x{}", hex::encode(tx_hash_input))).await {
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

    // Write RLP to file if we have the data
    if let Some(rlp_encodings) = result.get("rlp_encodings") {
        if let Some(eip2718_bytes) = rlp_encodings.get("eip2718_rlp_bytes") {
            if let Some(bytes_array) = eip2718_bytes.as_array() {
                let bytes: Vec<u8> = bytes_array.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                
                if let Err(e) = crate::transaction::write_rlp_to_file(&bytes, "tx_rlp_bytes.txt") {
                    eprintln!("Warning: Failed to write RLP to file: {}", e);
                }
            }
        }
    }

    result
}

/// Fetch block by block number
pub async fn fetch_block_by_number(rpc_url: &str, block_number: u64) -> Result<alloy::rpc::types::Block, String> {
    // Connect to provider
    let provider = connect_provider(rpc_url).await
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;
    
    // Fetch block details
    match provider.get_block_by_number(block_number.into()).full().await {
        Ok(Some(block)) => Ok(block),
        Ok(None) => Err("Block not found".to_string()),
        Err(e) => Err(format!("Error fetching block: {}", e)),
    }
} 