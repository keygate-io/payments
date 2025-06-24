use alloy::{primitives::keccak256};
use serde_json::{json, Value};
use alloy_trie::{proof::ProofRetainer, HashBuilder, Nibbles};
use crate::{rpc::fetch_block_by_number};
use hex;

mod trie_visualizer;
use trie_visualizer::visualize_trie;

/// Generate proof nodes by calling the Node.js script
/// TODO: This should be refactored to pure Rust implementation for WASM compatibility
pub async fn get_proof_nodes(block_number: u64, tx_hash: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let block = fetch_block_by_number(block_number).await?;

    let tx_hash_bytes = hex::decode(tx_hash.strip_prefix("0x").unwrap()).unwrap();

    // Need nibbles for target tx
    let target_index = block.transactions.hashes()
        .position(|tx| tx == tx_hash_bytes.as_slice())
        .ok_or("Transaction not found in block")?;

    let target_index_rlp = alloy_rlp::encode(target_index);
    let target_index_nibbles = Nibbles::unpack(keccak256(&target_index_rlp));

    let mut hb = HashBuilder::default().with_proof_retainer(ProofRetainer::new(vec![target_index_nibbles.clone()]));

    let iter = block.transactions.as_transactions();
    
    if iter.is_none() {
        return Err("Block has no transactions".into());
    }

    // Collect all transactions with their hashed keys
    let mut transactions: Vec<_> = iter.unwrap().iter().enumerate()
        .map(|(index, tx)| {
            let index_rlp = alloy_rlp::encode(index);
            let nibbles = Nibbles::unpack(keccak256(&index_rlp));
            (nibbles, tx)
        })
        .collect();

    // Sort by nibbles (keys)
    transactions.sort_by(|a, b| a.0.cmp(&b.0));

    // Now add them in sorted order
    for (nibbles, tx) in transactions {
        let mut tx_rlp = vec![];
        // Try each transaction type in order
        if let Some(tx_eip1559) = tx.inner.as_eip1559() {
            tx_rlp.push(0x02); // EIP-1559 type flag
            tx_eip1559.rlp_encode(&mut tx_rlp);
        } else if let Some(tx_legacy) = tx.inner.as_legacy() {
            tx_legacy.rlp_encode(&mut tx_rlp);
        } else if let Some(tx_eip2930) = tx.inner.as_eip2930() {
            tx_rlp.push(0x01); // EIP-2930 type flag
            tx_eip2930.rlp_encode(&mut tx_rlp);
        } else if let Some(tx_eip4844) = tx.inner.as_eip4844() {
            tx_rlp.push(0x03); // EIP-4844 type flag
            tx_eip4844.rlp_encode(&mut tx_rlp);
        } else if let Some(tx_eip7702) = tx.inner.as_eip7702() {
            tx_rlp.push(0x04); // EIP-7702 type flag
            tx_eip7702.rlp_encode(&mut tx_rlp);
        } else {
            println!("tx: {:?}", tx.inner);
            return Err("Unsupported transaction type".into());
        }
        
        hb.add_leaf(nibbles, &tx_rlp);
    }

    let proof_nodes = hb.take_proof_nodes();
    let proof_hex: Vec<String> = proof_nodes.iter()
        .map(|(nibbles, bytes)| format!("0x{}", hex::encode(bytes.as_ref())))
        .collect();

    let root = hb.root();

    Ok(json!({
        "success": true,
        "proof_nodes": proof_hex,
        "block_number": block_number,
        "root": format!("0x{}", hex::encode(root)),
        "tx_hash": tx_hash
    }))
}

/// Visualize the trie structure for a given transaction
pub async fn visualize_trie_for_transaction(block_number: u64, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let block = fetch_block_by_number(block_number).await?;

    let tx_hash_bytes = hex::decode(tx_hash.strip_prefix("0x").unwrap()).unwrap();

    // Need nibbles for target tx
    let target_index = block.transactions.hashes()
        .position(|tx| tx == tx_hash_bytes.as_slice())
        .ok_or("Transaction not found in block")?;

    let target_index_rlp = alloy_rlp::encode(target_index);
    let target_index_nibbles = Nibbles::unpack(keccak256(&target_index_rlp));

    let iter = block.transactions.as_transactions();
    
    if iter.is_none() {
        return Err("Block has no transactions".into());
    }

    // Collect all transactions with their hashed keys
    let mut transactions: Vec<_> = iter.unwrap().iter().enumerate()
        .map(|(index, tx)| {
            let index_rlp = alloy_rlp::encode(index);
            let nibbles = Nibbles::unpack(keccak256(&index_rlp));
            (nibbles, tx)
        })
        .collect();

    println!("Transaction count: {}", transactions.len());

    // Sort by nibbles (keys)
    transactions.sort_by(|a, b| a.0.cmp(&b.0));

    // Prepare transaction data for visualization
    let tx_data: Vec<_> = transactions.iter()
        .map(|(nibbles, tx)| {
            let mut tx_rlp = vec![];
            // Encode transaction (simplified version)
            if let Some(tx_legacy) = tx.inner.as_legacy() {
                tx_legacy.rlp_encode(&mut tx_rlp);
            } else if let Some(tx_eip1559) = tx.inner.as_eip1559() {
                tx_rlp.push(0x02); // EIP-1559 type flag
                tx_eip1559.rlp_encode(&mut tx_rlp);
            } else if let Some(tx_eip2930) = tx.inner.as_eip2930() {
                tx_rlp.push(0x01); // EIP-2930 type flag
                tx_eip2930.rlp_encode(&mut tx_rlp);
            } else if let Some(tx_eip4844) = tx.inner.as_eip4844() {
                tx_rlp.push(0x03); // EIP-4844 type flag
                tx_eip4844.rlp_encode(&mut tx_rlp);
            } else if let Some(tx_eip7702) = tx.inner.as_eip7702() {
                tx_rlp.push(0x04); // EIP-7702 type flag
                tx_eip7702.rlp_encode(&mut tx_rlp);
            }
            (nibbles.clone(), tx_rlp)
        })
        .collect();

    // Visualize the trie
    visualize_trie(&tx_data, Some(&target_index_nibbles));

    Ok(())
} 