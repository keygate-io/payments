import * as wasm from "../../core/pkg/keygate_core.js";

export interface TransactionDetails {
  hash: string;
  block_number?: number;
  transaction_index?: number;
  from: string;
  to?: string;
  value: string;
  gas: string;
  gas_price?: string;
  max_fee_per_gas?: string;
  max_priority_fee_per_gas?: string;
  input: string;
  nonce: string;
  transaction_type?: number;
  access_list?: any[];
  chain_id?: number;
  v?: string;
  r?: string;
  s?: string;
}

export interface ProofResult {
  success: boolean;
  proof_nodes?: any[];
  error?: string;
}

export class KeygateSDK {
  private client: wasm.KeygateClient;

  constructor(rpcUrl: string) {
    this.client = new wasm.KeygateClient(rpcUrl);
  }

  async fetchTransactionDetails(txHash: string): Promise<TransactionDetails> {
    try {
      const result = await this.client.fetch_transaction_details(txHash);
      return result as TransactionDetails;
    } catch (error) {
      throw new Error(`Failed to fetch transaction details: ${error}`);
    }
  }

  async getProofNodes(
    blockNumber: number,
    txHash: string
  ): Promise<ProofResult> {
    try {
      const result = await this.client.get_proof_nodes(
        BigInt(blockNumber),
        txHash
      );
      return result as ProofResult;
    } catch (error) {
      throw new Error(`Failed to get proof nodes: ${error}`);
    }
  }
}
