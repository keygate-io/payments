#!/usr/bin/env node

const { Trie } = require("@ethereumjs/trie");
const RLP = require("rlp");
const { ethers } = require("ethers");
const path = require("path");

// Load environment variables from parent directory
require("dotenv").config({ path: path.join(__dirname, "..", ".env") });

// Configuration from environment variables (similar to main.rs)
const RPC_URL = process.env.RPC_URL || "http://localhost:8545";
const DEFAULT_TX_HASH =
  process.env.DEFAULT_TX_HASH ||
  "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";

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
 * @param {string} rpcUrl - Optional RPC URL (defaults to environment variable)
 * @returns {Object} Object containing txIndex, proofNodes, and blockTxRoot
 */
async function getProofNodes(blockNumber, targetTxHash, rpcUrl = RPC_URL) {
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

      console.error("Using this transaction format for RLP encoding", txData);

      // RLP encode the transaction
      const rlpEncodedTx = RLP.encode(txData);
      const rlpBytes = Buffer.from(rlpEncodedTx);

      console.error(
        `RLP hash of transaction ${tx.hash}: `,
        "0x" + rlpBytes.toString("hex")
      );

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
  } catch (error) {
    return false;
  }
}

/**
 * Get RLP encoding data for a specific transaction
 * @param {string} txHash - The transaction hash to get RLP data for
 * @param {string} rpcUrl - Optional RPC URL (defaults to environment variable)
 * @returns {Object} Object containing RLP hash and bytes array
 */
async function getRLPData(txHash, rpcUrl = RPC_URL) {
  try {
    console.error(`Fetching RLP data for transaction ${txHash}`);
    console.error(`Using RPC URL: ${rpcUrl}`);

    // Initialize provider
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);

    // Fetch transaction
    const tx = await provider.getTransaction(txHash);

    if (!tx) {
      throw new Error(`Transaction ${txHash} not found`);
    }

    console.error("Transaction found, preparing RLP encoding...");

    // Prepare transaction data for RLP encoding
    const txData = prepareTxForRLP(tx);

    console.error("Using this transaction format for RLP encoding", txData);

    // RLP encode the transaction
    const rlpEncodedTx = RLP.encode(txData);
    const rlpBytes = Buffer.from(rlpEncodedTx);

    // Calculate keccak256 hash of RLP bytes
    const rlpHash = ethers.utils.keccak256(rlpBytes);

    const result = {
      success: true,
      txHash: txHash,
      rlpHash: rlpHash,
      rlpBytes: "0x" + rlpBytes.toString("hex"),
      rlpBytesArray: Array.from(rlpBytes),
      rlpLength: rlpBytes.length,
    };

    return result;
  } catch (error) {
    return {
      success: false,
      error: error.message,
      txHash: txHash,
    };
  }
}

async function main() {
  const txHash =
    "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";
  const result = await getRLPData(txHash);
  console.log("Transaction RLP Result:", JSON.stringify(result, null, 2));

  // Get the proof nodes for the transaction
  const proofNodes = await getProofNodes(22670282, txHash);
  console.log("Proof Nodes:", JSON.stringify(proofNodes, null, 2));
}

// Export functions for module usage
module.exports = {
  getProofNodes,
  verifyTransactionProof,
  getRLPData,
};

// Run CLI if this file is executed directly
if (require.main === module) {
  main().catch(console.error);
}
