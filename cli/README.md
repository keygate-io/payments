# Keygate CLI (TypeScript)

A TypeScript CLI equivalent of the Rust keygate tool for ZK event payment verification.

## Setup

1. Install dependencies:
```bash
cd cli
npm install
```

2. Build the project:
```bash
npm run build
```

3. Link the CLI globally (optional):
```bash
npm link
```

## Usage

The CLI provides the same commands as the Rust version:

### Prepare Command
```bash
keygate prepare <transaction_hash> <expected_to> <expected_value> [--debug]
```

### RLP Encoding Command
```bash
keygate rlp-of <transaction_hash> [--debug]
```

### Proof Generation Command
```bash
keygate proof <transaction_hash> [--debug]
```

### Trie Visualization Command
```bash
keygate visualize-nibble-trie <transaction_hash> [--debug]
```

## Development

### Running in development mode
```bash
npm run dev -- <command> [args]
```

### Building
```bash
npm run build
```

### Testing
```bash
npm test
```

## WASM Integration

**Important**: This CLI is designed to use WebAssembly bindings from the core Rust module. Currently, the WASM bindings need to be properly configured with wasm-bindgen exports.

To set up WASM integration:

1. Add wasm-bindgen attributes to the Rust functions in `core/src/lib.rs`
2. Build the WASM module with proper exports
3. Update the WASM import in `src/wasm.ts`

Example of required Rust exports:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn fetch_transaction_details(tx_hash: &str) -> Result<JsValue, JsValue> {
    // Implementation
}

#[wasm_bindgen]
pub async fn process_transaction_rlp(tx: &JsValue) -> Result<JsValue, JsValue> {
    // Implementation
}

// ... other exports
```

## Architecture

- `src/index.ts` - Main CLI entry point with command parsing
- `src/commands/` - Individual command handlers
- `src/wasm.ts` - WASM integration layer with TypeScript types
- `dist/` - Compiled JavaScript output 