#!/usr/bin/env node

const { Trie } = require("@ethereumjs/trie");
const RLP = require("rlp");
const { ethers } = require("ethers");

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
async function getProofNodes(blockNumber, targetTxHash, rpcUrl) {
  if (!rpcUrl) {
    throw new Error("RPC_URL is required");
  }

  try {
    // Initialize provider
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);

    // Fetch block with full transactions
    const block = await provider.getBlockWithTransactions(blockNumber);

    if (!block) {
      throw new Error(`Block ${blockNumber} not found`);
    }

    // Create new trie
    const trie = new Trie();

    // Build the transaction trie locally
    for (let i = 0; i < block.transactions.length; i++) {
      const tx = block.transactions[i];

      // Prepare transaction data for RLP encoding
      const txData = prepareTxForRLP(tx);

      // RLP encode the transaction
      const rlpEncodedTx = RLP.encode(txData);
      const rlpBytes = Buffer.from(rlpEncodedTx);

      // Use transaction index as key (RLP encoded)
      const key = RLP.encode(i);

      // Put in trie
      await trie.put(Buffer.from(key), Buffer.from(rlpEncodedTx));
    }

    // Find the index of the target transaction
    const txIndex = block.transactions.findIndex(
      (tx) => tx.hash.toLowerCase() === targetTxHash.toLowerCase()
    );

    if (txIndex === -1) {
      throw new Error(
        `Transaction ${targetTxHash} not found in block ${blockNumber}`
      );
    }

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
 * @param {string} rpcUrl - RPC URL (required)
 * @returns {Object} Object containing RLP hash and bytes array
 */
async function getRLPData(txHash, rpcUrl) {
  if (!rpcUrl) {
    throw new Error("RPC_URL is required");
  }

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

// Export functions for module usage
module.exports = {
  getProofNodes,
  verifyTransactionProof,
  getRLPData,
};

// Run CLI if this file is executed directly
if (require.main === module) {
  console.log("Not a script");
}
