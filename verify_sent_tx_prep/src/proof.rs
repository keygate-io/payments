use std::process::Command;
use alloy::{consensus::Transaction, primitives::keccak256};
use serde_json::{json, Value};
use alloy_trie::{proof::ProofRetainer, HashBuilder, Nibbles};
use alloy_rlp::BufMut;
use crate::{rpc::fetch_block_by_number};
use hex;

/// Generate proof nodes by calling the Node.js script
/// TODO: This should be refactored to pure Rust implementation for WASM compatibility
pub async fn get_proof_nodes(block_number: u64, tx_hash: &str) -> Result<Value, Box<dyn std::error::Error>> {

    let mut hb = HashBuilder::default().with_proof_retainer(ProofRetainer::default());

    let block = fetch_block_by_number(block_number).await?;

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
    // Transactions in this block can be Legacy, EIP-1559, EIP-2930, EIP-

        println!("Printing tx inner");
        println!("Tx inner: {:?}", tx.inner);
        let tx_as_eip1559 = tx.inner.as_eip1559().unwrap();
        let mut tx_rlp = vec![];
        tx_as_eip1559.rlp_encode(&mut tx_rlp);
        hb.add_leaf(nibbles, &tx_rlp);
    }

    let proof_nodes = hb.take_proof_nodes();
    let proof_hex: Vec<String> = proof_nodes.iter()
        .map(|(nibbles, bytes)| format!("0x{}", hex::encode(bytes.as_ref())))
        .collect();

    Ok(json!({
        "success": true,
        "proof_nodes": proof_hex,
        "block_number": block_number,
        "tx_hash": tx_hash
    }))
} 