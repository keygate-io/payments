use alloy::{
    primitives::{B256, keccak256}, 
    providers::{Provider, ProviderBuilder}
};
use hex;
use std::str::FromStr;

fn get_rpc_url() -> String {
    dotenvy::var("RPC_URL")
        .unwrap_or_else(|_| "https://mainnet.infura.io/v3/YOUR_INFURA_KEY_HERE".to_string())
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

fn verify_eip2718_hash(consensus_tx: &alloy::consensus::TxEnvelope, expected_hash: B256) -> bool {
    let (_, eip2718_bytes) = get_rlp_encodings(consensus_tx);
    let eip2718_hash = keccak256(&eip2718_bytes);
    eip2718_hash.as_slice() == expected_hash.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eip2718_hash_verification() {
        let rpc_url = get_rpc_url();
        let provider = ProviderBuilder::new().connect(&rpc_url).await.unwrap();
        
        // Test with a known transaction hash
        let tx_hash = B256::from_str("0x9268f692c8019779077f2bb383cf81b4ff069799db17afb12dbc4f5638f85f5f").unwrap(); // Replace with a real transaction hash
        
        match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => {
                let consensus_tx: alloy::consensus::TxEnvelope = tx.try_into().unwrap();
                assert!(verify_eip2718_hash(&consensus_tx, tx_hash), 
                    "EIP-2718 hash verification failed for transaction {}", 
                    hex::encode(tx_hash));
            }
            Ok(None) => panic!("Transaction not found"),
            Err(e) => panic!("Error fetching transaction: {}", e),
        }
    }
} 