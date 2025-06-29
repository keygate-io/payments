use wasm_bindgen::prelude::*;

pub mod transaction;
pub mod proof;
pub mod utils;
pub mod rpc;

// Re-export main functionality for easy access
pub use transaction::{get_rlp_encodings, write_rlp_to_file, process_transaction_rlp};
pub use proof::get_proof_nodes;
pub use utils::{index_to_nibbles, format_nibble_path};
pub use rpc::{connect_provider, fetch_transaction_details, process_full_transaction};

#[wasm_bindgen]
pub struct KeygateClient {
    rpc_url: String,
}

// Enable console.log! for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro for console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// WASM exports for JavaScript/TypeScript
#[wasm_bindgen]
impl KeygateClient {
    #[wasm_bindgen(constructor)]
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    #[wasm_bindgen]
    pub async fn fetch_transaction_details(&self, tx_hash: &str) -> Result<JsValue, JsValue> {
        match rpc::fetch_transaction_details(&self.rpc_url, tx_hash).await {
            Ok(tx) => {
                match serde_wasm_bindgen::to_value(&tx) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                }
            }
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }
    
    #[wasm_bindgen]
    pub async fn get_proof_nodes(&self, block_number: u64, tx_hash: &str) -> Result<JsValue, JsValue> {
        match proof::get_proof_nodes(&self.rpc_url, block_number, tx_hash).await {
            Ok(result) => {
                match serde_wasm_bindgen::to_value(&result) {
                    Ok(js_value) => Ok(js_value),
                    Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                }
            }
            Err(e) => Err(JsValue::from_str(&format!("Error getting proof nodes: {}", e))),
        }
    }
} 