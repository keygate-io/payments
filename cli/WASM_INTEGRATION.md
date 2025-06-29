# WASM Integration Guide

This document explains how to complete the integration between the TypeScript CLI and the Rust core WASM module.

## Current Status

The TypeScript CLI structure is complete and functional, but the WASM bindings are not yet properly configured. The CLI will show a warning message when run.

## Required Changes

### 1. Update Rust Core for WASM Export

Add wasm-bindgen attributes to `core/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Enable console.log! for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define JS-compatible types
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct TransactionDetails {
    pub hash: String,
    pub block_number: Option<u64>,
    pub transaction_index: Option<u64>,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas: String,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub input: String,
    pub nonce: String,
    pub transaction_type: Option<u8>,
    pub chain_id: Option<u64>,
    pub v: Option<String>,
    pub r: Option<String>,
    pub s: Option<String>,
}

// Export async functions for WASM
#[wasm_bindgen]
pub async fn fetch_transaction_details(tx_hash: &str) -> Result<JsValue, JsValue> {
    match crate::rpc::fetch_transaction_details(tx_hash).await {
        Ok(tx) => Ok(serde_wasm_bindgen::to_value(&tx)?),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

#[wasm_bindgen]
pub async fn process_transaction_rlp(tx_js: &JsValue) -> Result<JsValue, JsValue> {
    let tx: crate::transaction::TransactionDetails = serde_wasm_bindgen::from_value(tx_js.clone())?;
    match crate::transaction::process_transaction_rlp(&tx) {
        Ok(result) => Ok(serde_wasm_bindgen::to_value(&result)?),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

#[wasm_bindgen]
pub async fn process_full_transaction(
    tx_hash: &str,
    expected_to: &str,
    expected_value: &str,
) -> Result<JsValue, JsValue> {
    match crate::rpc::process_full_transaction(tx_hash, expected_to, expected_value).await {
        Ok(result) => Ok(serde_wasm_bindgen::to_value(&result)?),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

#[wasm_bindgen]
pub async fn get_proof_nodes(block_number: u64, tx_hash: &str) -> Result<JsValue, JsValue> {
    match crate::proof::get_proof_nodes(block_number, tx_hash).await {
        Ok(result) => Ok(serde_wasm_bindgen::to_value(&result)?),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

#[wasm_bindgen]
pub async fn visualize_trie_for_transaction(block_number: u64, tx_hash: &str) -> Result<(), JsValue> {
    match crate::proof::visualize_trie_for_transaction(block_number, tx_hash).await {
        Ok(_) => Ok(()),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}
```

### 2. Update Cargo.toml

Add WASM dependencies to `core/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
wasm-bindgen = "0.2"
serde-wasm-bindgen = "0.6"
js-sys = "0.3"
web-sys = "0.3"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]
```

### 3. Build WASM Module

```bash
cd core
wasm-pack build --target nodejs --out-dir pkg
```

### 4. Update TypeScript WASM Integration

Once the WASM module is properly built, update `cli/src/wasm.ts`:

```typescript
import * as wasm from '@/core';

// Remove placeholder implementations and use actual WASM functions
export async function fetchTransactionDetails(txHash: string): Promise<TransactionDetails> {
  const result = await wasm.fetch_transaction_details(txHash);
  return result;
}

export async function processTransactionRlp(tx: TransactionDetails): Promise<ProcessTransactionResult> {
  const result = await wasm.process_transaction_rlp(tx);
  return result;
}

// ... etc for other functions
```

## Testing

After implementing the changes:

1. Build the WASM module: `cd core && wasm-pack build --target nodejs`
2. Build the CLI: `cd cli && npm run build`
3. Test with a real transaction hash: `node dist/index.js rlp-of 0x...`

## File Structure After Integration

```
cli/
├── src/
│   ├── index.ts              # Main CLI entry point
│   ├── wasm.ts               # WASM integration layer
│   └── commands/             # Command handlers
├── dist/                     # Compiled output
└── node_modules/            # Dependencies

core/
├── src/
│   ├── lib.rs               # WASM exports
│   └── ...                  # Other modules
└── pkg/                     # Generated WASM bindings
    ├── keygate.js           # JS bindings
    ├── keygate.d.ts         # TypeScript definitions
    └── keygate_bg.wasm      # WASM binary
``` 