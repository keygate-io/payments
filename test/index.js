import { UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import { Trie } from "@ethereumjs/trie";
import RLP from "rlp";
import { ethers } from "ethers";
import { Buffer } from "buffer";
import circuit from "./circuits/payment_verify.json" with { type: "json" };

// Configuration - can be overridden by passing rpcUrl parameter
const DEFAULT_RPC_URL =
  "https://mainnet.infura.io/v3/ba2572c3cedd43deaa43fd9e00261c33";

/**
 * Convert transaction to RLP-encodable format
 * @param {Object} tx - Transaction object from ethers
 * @returns {Array} Array of properly formatted transaction fields
 */
function prepareTxForRLP(tx) {
  // Convert BigNumber and other objects to hex strings/bytes
  return [
    ethers.utils.hexlify(tx.nonce || 0),
    ethers.utils.hexlify(tx.gasPrice || 0),
    ethers.utils.hexlify(tx.gasLimit || 0),
    tx.to || "0x", // null for contract creation becomes empty string
    ethers.utils.hexlify(tx.value || 0),
    tx.data || "0x",
    ethers.utils.hexlify(tx.v || 0),
    tx.r || "0x",
    tx.s || "0x",
  ];
}

/**
 * Get proof nodes for a specific transaction in a block
 * @param {number} blockNumber - The block number to fetch
 * @param {string} targetTxHash - The transaction hash to create proof for
 * @param {string} rpcUrl - RPC URL (required)
 * @returns {Object} Object containing txIndex, proofNodes, and blockTxRoot
 */
async function getProofNodes(
  blockNumber,
  targetTxHash,
  rpcUrl = DEFAULT_RPC_URL
) {
  try {
    console.error(
      `Fetching proof nodes for tx ${targetTxHash} in block ${blockNumber}`
    );
    console.error(`Using RPC URL: ${rpcUrl}`);

    // Initialize provider
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);

    // Fetch block with full transactions
    console.error("Fetching block with transactions...");
    const block = await provider.getBlockWithTransactions(blockNumber);

    if (!block) {
      throw new Error(`Block ${blockNumber} not found`);
    }

    console.error(`Block found with ${block.transactions.length} transactions`);

    // Create new trie
    const trie = new Trie();

    // Build the transaction trie locally
    console.error("Building transaction trie...");
    for (let i = 0; i < block.transactions.length; i++) {
      const tx = block.transactions[i];

      // Prepare transaction data for RLP encoding
      const txData = prepareTxForRLP(tx);

      console.log("Using this transaction format for RLP encoding", txData);

      // RLP encode the transaction
      const rlpEncodedTx = RLP.encode(txData);

      // Use transaction index as key (RLP encoded)
      const key = RLP.encode(i);

      // Put in trie
      await trie.put(Buffer.from(key), Buffer.from(rlpEncodedTx));
    }

    console.error("Transaction trie built successfully");

    // Find the index of the target transaction
    const txIndex = block.transactions.findIndex(
      (tx) => tx.hash.toLowerCase() === targetTxHash.toLowerCase()
    );

    if (txIndex === -1) {
      throw new Error(
        `Transaction ${targetTxHash} not found in block ${blockNumber}`
      );
    }

    console.error(`Target transaction found at index ${txIndex}`);

    // Create proof for the transaction
    const key = RLP.encode(txIndex);
    const proof = await trie.createProof(Buffer.from(key));

    const result = {
      success: true,
      txIndex,
      proofNodes: proof.map((node) => "0x" + Buffer.from(node).toString("hex")),
      blockTxRoot: "0x" + Buffer.from(trie.root()).toString("hex"),
      blockNumber: block.number,
      txHash: targetTxHash,
      totalTransactions: block.transactions.length,
    };

    return result;
  } catch (error) {
    return {
      success: false,
      error: error.message,
      txHash: targetTxHash,
      blockNumber,
    };
  }
}

/**
 * Verify a transaction proof against a block's transaction root
 * @param {Array} proofNodes - Array of proof node hex strings
 * @param {number} txIndex - Index of the transaction in the block
 * @param {string} blockTxRoot - The block's transaction root
 * @returns {boolean} True if proof is valid
 */
async function verifyTransactionProof(proofNodes, txIndex, blockTxRoot) {
  try {
    // Convert proof nodes from hex strings to buffers
    const proof = proofNodes.map((node) =>
      Buffer.from(node.replace("0x", ""), "hex")
    );

    // Create key for the transaction index
    const key = Buffer.from(RLP.encode(txIndex));

    // Verify the proof using trie instance
    const trie = new Trie();
    const value = await trie.verifyProof(
      Buffer.from(blockTxRoot.replace("0x", ""), "hex"),
      key,
      proof
    );

    return !!value;
  } catch {
    return false;
  }
}

/**
 * Get transaction RLP from transaction hash
 * @param {string} transactionHash - The transaction hash
 * @param {string} rpcUrl - RPC URL (required)
 * @returns {Object} Object containing RLP encoded transaction and additional data
 */
async function getTransactionRLP(transactionHash, rpcUrl = DEFAULT_RPC_URL) {
  try {
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);

    // Get transaction details
    const tx = await provider.getTransaction(transactionHash);
    if (!tx) {
      throw new Error(`Transaction ${transactionHash} not found`);
    }

    // Prepare transaction for RLP encoding
    const txData = prepareTxForRLP(tx); // OK

    console.log("Using this transaction format for RLP encoding", txData);

    // RLP encode the transaction
    const rlpEncodedTx = RLP.encode(txData); // OK

    console.log("RLP encoded transaction", rlpEncodedTx);

    const rlpBytes = Buffer.from(rlpEncodedTx);

    console.log(
      `RLP hash of transaction ${transactionHash}: `,
      "0x" + rlpBytes.toString("hex")
    ); // OK

    console.log(
      `RLP bytes of transaction ${transactionHash}: `,
      rlpBytes.toString("hex")
    ); // NOT OK

    console.log(
      `RLP bytes of transaction ${transactionHash} (as array): `,
      Array.from(rlpBytes)
    );

    return {
      success: true,
      rlp: "0x" + rlpBytes.toString("hex"),
      rlpBytes: rlpBytes,
      transaction: tx,
      txData,
    };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      transactionHash,
    };
  }
}

async function proofOfPayment({
  expected_to,
  expected_value,
  transaction_hash,
  rpcUrl = DEFAULT_RPC_URL,
}) {
  try {
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode);

    // Get transaction RLP from transaction_hash
    const rlpResult = await getTransactionRLP(transaction_hash, rpcUrl);

    console.log("RLP result", rlpResult);

    if (!rlpResult.success) {
      throw new Error(`Failed to get transaction RLP: ${rlpResult.error}`);
    }

    // Verify transaction matches expected values
    const tx = rlpResult.transaction;
    if (tx.to?.toLowerCase() !== expected_to.toLowerCase()) {
      throw new Error(
        `Transaction recipient ${tx.to} does not match expected ${expected_to}`
      );
    }

    console.error(`Transaction value: ${tx.value}`);
    if (tx.value.toString() !== expected_value.toString()) {
      throw new Error(
        `Transaction value ${tx.value} does not match expected ${expected_value}`
      );
    }

    // Convert Buffer to array of numbers and pad to 256 bytes
    const rlpArray = Array.from(rlpResult.rlpBytes);
    const paddedRlpArray = new Array(256).fill(0);

    // Copy the RLP bytes to the beginning of the padded array
    for (let i = 0; i < Math.min(rlpArray.length, 256); i++) {
      paddedRlpArray[i] = rlpArray[i];
    }

    console.log("Padded RLP array", paddedRlpArray);

    // Prepare inputs for the Noir circuit
    const inputs = {
      tx_rlp: paddedRlpArray,
      expected_to: expected_to,
      expected_value: expected_value,
      transaction_hash: transaction_hash,
    };

    // Generate the proof using Noir
    const { witness } = await noir.execute(inputs);

    console.log("Witness", witness);

    const proof = await backend.generateProof(witness);

    return {
      success: true,
      proof: proof,
      expected_to,
      expected_value,
      transaction_hash,
      rlp: rlpResult.rlp,
    };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      expected_to,
      expected_value,
      transaction_hash,
    };
  }
}

export {
  proofOfPayment,
  getProofNodes,
  verifyTransactionProof,
  getTransactionRLP,
};

async function main() {
  const txHash =
    "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";
  const result = await getTransactionRLP(txHash);
  console.log("Transaction RLP Result:", JSON.stringify(result, null, 2));
}

main().catch(console.error);
