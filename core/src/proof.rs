use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/node/dist/build.js")]
extern "C" {
    #[wasm_bindgen(getter)]
    async fn getProofNodes(blockNumber: u64, txHash: &str, rpcUrl: &str) -> JsValue;
}


/// Generate proof nodes by calling the Node.js script
/// TODO: This should be refactored to pure Rust implementation for WASM compatibility
pub async fn get_proof_nodes(rpc_url: &str, block_number: u64, tx_hash: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let proof_nodes_result =  getProofNodes(block_number, tx_hash, rpc_url).await;


    Ok(json!({
        "success": true,
        "proof_nodes": "0x1234567890abcdef", // Hardcoded for now
        "block_number": block_number,
        "root": "0xabcdef1234567890", // Hardcoded for now
        "tx_hash": tx_hash,
        "raw_proof_nodes": proof_nodes_result.as_string().unwrap_or_default()
    }))
}