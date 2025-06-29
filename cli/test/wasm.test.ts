import { describe, it, expect, beforeAll } from "vitest";
import { KeygateSDK } from "../src/sdk";
import dotenv from "dotenv";

dotenv.config();

describe("Keygate SDK Proof Nodes", () => {
  let sdk: KeygateSDK;

  beforeAll(async () => {
    // Use a custom RPC URL or fallback to environment variable
    const rpcUrl =
      process.env.RPC_URL || "https://eth-mainnet.g.alchemy.com/v2/demo";
    sdk = new KeygateSDK(rpcUrl);
  });

  it("should get proof nodes for the specified transaction and block", async () => {
    console.log("Getting proof nodes for the specified transaction and block");
    const txHash =
      "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";
    const blockNumber = 22670282;

    const proofResult = await sdk.getProofNodes(blockNumber, txHash);

    console.log(proofResult);

    expect(proofResult).toBeDefined();
    expect(proofResult.success).toBe(true);
    expect(proofResult.proof_nodes).toBeDefined();
    expect(Array.isArray(proofResult.proof_nodes)).toBe(true);
    expect(proofResult.proof_nodes!.length).toBeGreaterThan(0);
  });
});
