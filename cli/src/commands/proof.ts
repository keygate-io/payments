import { KeygateSDK } from "@keygate/sdk";

export async function proofCommand(
  transactionHash: string,
  debug: boolean
): Promise<void> {
  try {
    // Initialize SDK with RPC URL
    const rpcUrl =
      process.env.RPC_URL || "https://eth-mainnet.g.alchemy.com/v2/demo";
    const sdk = new KeygateSDK(rpcUrl);

    // Get transaction details to find block number
    const tx = await sdk.fetchTransactionDetails(transactionHash);

    if (!tx.block_number) {
      throw new Error("Transaction not yet included in a block");
    }

    if (debug) {
      console.error(`Debug: Processing transaction ${transactionHash}`);
      console.error(`Debug: Block number: ${tx.block_number}`);
    }

    // Generate proof nodes
    const result = await sdk.getProofNodes(tx.block_number, transactionHash);

    console.log(JSON.stringify(result, null, 2));
  } catch (error) {
    throw new Error(
      `Failed to generate proof: ${
        error instanceof Error ? error.message : String(error)
      }`
    );
  }
}
