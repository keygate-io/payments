import { describe, it, expect, beforeAll } from "vitest";
import { getProofNodes } from "../index.js";
import dotenv from "dotenv";

dotenv.config();

describe("Proof Nodes", () => {
  let rpcUrl;

  beforeAll(async () => {
    // Use a custom RPC URL or fallback to environment variable
    rpcUrl = process.env.RPC_URL || "https://eth-mainnet.g.alchemy.com/v2/demo";
  });

  it("should get proof nodes for the specified transaction and block", async () => {
    console.log("Getting proof nodes for the specified transaction and block");
    const txHash =
      "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";
    const blockNumber = 22670282;

    const proofResult = await getProofNodes(blockNumber, txHash, rpcUrl);

    console.log("Proof nodes", proofResult.proofNodes);

    expect(proofResult).toBeDefined();
    expect(proofResult.success).toBe(true);
    expect(proofResult.proofNodes).toBeDefined();
    expect(Array.isArray(proofResult.proofNodes)).toBe(true);
    expect(proofResult.proofNodes.length).toBeGreaterThan(0);
    expect(proofResult.txIndex).toBeDefined();
    expect(typeof proofResult.txIndex).toBe("number");
    expect(proofResult.blockTxRoot).toBeDefined();
    expect(proofResult.blockNumber).toBe(blockNumber);
    expect(proofResult.txHash).toBe(txHash);
    expect(proofResult.totalTransactions).toBeDefined();
    expect(typeof proofResult.totalTransactions).toBe("number");
  });
});
