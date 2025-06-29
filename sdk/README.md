# @keygate/sdk

SDK for Keygate - tools for building on-chain payment applications.

## Installation

Install the package using npm:

```bash
npm install @keygate/sdk
```

## Usage

Here's a basic example of how to use the SDK to fetch transaction details and generate proof nodes.

```typescript
import { KeygateSDK } from "@keygate/sdk";

async function main() {
  // Initialize the SDK with an Ethereum RPC URL
  const rpcUrl = "https://eth-mainnet.g.alchemy.com/v2/your-api-key";
  const sdk = new KeygateSDK(rpcUrl);

  const transactionHash = "0x..."; // Your transaction hash here

  try {
    // 1. Fetch transaction details to get the block number
    const txDetails = await sdk.fetchTransactionDetails(transactionHash);

    if (!txDetails.block_number) {
      console.error("Transaction is not yet mined.");
      return;
    }

    // 2. Generate the proof nodes
    const proofResult = await sdk.getProofNodes(
      txDetails.block_number,
      transactionHash
    );

    if (proofResult.success) {
      console.log("Proof generated successfully:");
      console.log(JSON.stringify(proofResult.proof_nodes, null, 2));
    } else {
      console.error("Failed to generate proof:", proofResult.error);
    }
  } catch (error) {
    console.error("An error occurred:", error);
  }
}

main();
```

## API Reference

### `new KeygateSDK(rpcUrl: string)`

Creates a new instance of the Keygate SDK.

-   `rpcUrl`: The URL of an Ethereum JSON-RPC endpoint.

### `async fetchTransactionDetails(txHash: string): Promise<TransactionDetails>`

Fetches the details of a specific transaction from the blockchain.

-   `txHash`: The hash of the transaction to fetch.
-   Returns: A promise that resolves to a `TransactionDetails` object.

### `async getProofNodes(blockNumber: number, txHash: string): Promise<ProofResult>`

Generates the proof nodes required to verify a transaction.

-   `blockNumber`: The number of the block containing the transaction.
-   `txHash`: The hash of the transaction.
-   Returns: A promise that resolves to a `ProofResult` object.
