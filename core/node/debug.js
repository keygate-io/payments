const { ethers } = require("ethers");
require("dotenv").config({ path: "../.env" });

async function checkTransaction() {
  const provider = new ethers.providers.JsonRpcProvider(process.env.RPC_URL);
  const txHash =
    "0xf6af2b8cd59e6f7c6a9667f5ef81cfb5438a05a161fca7a1cac5aa0f1ae54422";
  const blockNumber = 22674559;

  // Get transaction details
  const tx = await provider.getTransaction(txHash);
  console.log("Transaction type:", tx.type);
  console.log("Transaction details:");
  console.log("  nonce:", tx.nonce);
  console.log("  gasPrice:", tx.gasPrice?.toString());
  console.log("  maxFeePerGas:", tx.maxFeePerGas?.toString());
  console.log("  maxPriorityFeePerGas:", tx.maxPriorityFeePerGas?.toString());
  console.log("  gasLimit:", tx.gasLimit?.toString());
  console.log("  to:", tx.to);
  console.log("  value:", tx.value?.toString());
  console.log("  data length:", tx.data?.length);
  console.log("  accessList:", tx.accessList);

  // Get block and check transaction root
  const block = await provider.getBlock(blockNumber);
  console.log("\nBlock transaction root:", block.transactionsRoot);

  // Check if we can get the raw transaction
  try {
    const rawTx = await provider.send("eth_getRawTransactionByHash", [txHash]);
    console.log("\nRaw transaction:", rawTx);
  } catch (e) {
    console.log("Could not get raw transaction:", e.message);
  }
}

checkTransaction().catch(console.error);
