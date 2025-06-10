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
    console.log(
      `Fetching proof nodes for tx ${targetTxHash} in block ${blockNumber}`
    );
    console.log(`Using RPC URL: ${rpcUrl}`);

    // Initialize provider
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);

    // Fetch block with full transactions
    console.log("Fetching block with transactions...");
    const block = await provider.getBlockWithTransactions(blockNumber);

    if (!block) {
      throw new Error(`Block ${blockNumber} not found`);
    }

    console.log(`Block found with ${block.transactions.length} transactions`);

    // Create new trie
    const trie = new Trie();

    // Build the transaction trie locally
    console.log("Building transaction trie...");
    for (let i = 0; i < block.transactions.length; i++) {
      const tx = block.transactions[i];

      // Prepare transaction data for RLP encoding
      const txData = prepareTxForRLP(tx);

      // RLP encode the transaction
      const rlpEncodedTx = RLP.encode(txData);

      // Use transaction index as key (RLP encoded)
      const key = RLP.encode(i);

      // Put in trie
      await trie.put(Buffer.from(key), Buffer.from(rlpEncodedTx));
    }

    console.log("Transaction trie built successfully");

    // Find the index of the target transaction
    const txIndex = block.transactions.findIndex(
      (tx) => tx.hash.toLowerCase() === targetTxHash.toLowerCase()
    );

    if (txIndex === -1) {
      throw new Error(
        `Transaction ${targetTxHash} not found in block ${blockNumber}`
      );
    }

    console.log(`Target transaction found at index ${txIndex}`);

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

function printUsage() {
  console.log("Usage: node index.js <command> [options]");
  console.log("");
  console.log("Commands:");
  console.log(
    "  proof <block_number> <tx_hash>     Generate proof nodes for a transaction"
  );
  console.log(
    "  verify <tx_index> <block_tx_root> <proof_node1> [proof_node2] ...    Verify proof nodes"
  );
  console.log(
    "  proof-and-verify <block_number> <tx_hash>    Generate proof and immediately verify it"
  );
  console.log("");
  console.log("Examples:");
  console.log("  node index.js proof 19000000 0x1234...");
  console.log("  node index.js verify 5 0xabcd... 0x1234... 0x5678...");
  console.log("  node index.js proof-and-verify 19000000 0x1234...");
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    printUsage();
    process.exit(1);
  }

  const command = args[0];

  try {
    switch (command) {
      case "proof": {
        if (args.length < 3) {
          console.error(
            "Error: proof command requires block number and transaction hash"
          );
          printUsage();
          process.exit(1);
        }

        const blockNumber = parseInt(args[1]);
        const txHash = args[2];

        if (isNaN(blockNumber)) {
          console.error("Error: Block number must be a valid integer");
          process.exit(1);
        }

        const result = await getProofNodes(blockNumber, txHash);

        // Output JSON result to stdout
        console.log(JSON.stringify(result, null, 2));

        if (!result.success) {
          process.exit(1);
        }
        break;
      }

      case "verify": {
        if (args.length < 4) {
          console.error(
            "Error: verify command requires tx_index, block_tx_root, and at least one proof node"
          );
          printUsage();
          process.exit(1);
        }

        const txIndex = parseInt(args[1]);
        const blockTxRoot = args[2];
        const proofNodes = args.slice(3);

        if (isNaN(txIndex)) {
          console.error("Error: Transaction index must be a valid integer");
          process.exit(1);
        }

        const isValid = await verifyTransactionProof(
          proofNodes,
          txIndex,
          blockTxRoot
        );

        console.log("isValid", isValid);

        const result = {
          success: true,
          isValid,
          txIndex,
          blockTxRoot,
          proofNodesCount: proofNodes.length,
        };

        console.log(JSON.stringify(result, null, 2));
        break;
      }

      case "proof-and-verify": {
        if (args.length < 3) {
          console.error(
            "Error: proof-and-verify command requires block number and transaction hash"
          );
          printUsage();
          process.exit(1);
        }

        const blockNumber = parseInt(args[1]);
        const txHash = args[2];

        if (isNaN(blockNumber)) {
          console.error("Error: Block number must be a valid integer");
          process.exit(1);
        }

        const result = await getProofNodes(blockNumber, txHash);

        if (!result.success) {
          console.error("Error: Unable to generate proof");
          process.exit(1);
        }

        const isValid = await verifyTransactionProof(
          result.proofNodes,
          result.txIndex,
          result.blockTxRoot
        );

        const resultWithVerification = {
          ...result,
          isValid,
        };

        console.log(JSON.stringify(resultWithVerification, null, 2));
        break;
      }

      default:
        console.error(`Error: Unknown command '${command}'`);
        printUsage();
        process.exit(1);
    }
  } catch (error) {
    const errorResult = {
      success: false,
      error: error.message,
    };
    console.log(JSON.stringify(errorResult, null, 2));
    process.exit(1);
  }
}

// Export functions for module usage
module.exports = {
  getProofNodes,
  verifyTransactionProof,
};

// Run CLI if this file is executed directly
if (require.main === module) {
  main();
}
