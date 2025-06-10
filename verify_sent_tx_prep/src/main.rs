use std::io::{self, Write};
use std::str::FromStr;
use alloy::eips::BlockNumberOrTag;
use alloy::{
    primitives::{B256, keccak256, Address}, 
    providers::{Provider, ProviderBuilder}
};
use hex;
use alloy_rlp::{encode};
use alloy_trie::{HashBuilder, Nibbles};

const RPC_URL: &str = "https://mainnet.infura.io/v3/ba2572c3cedd43deaa43fd9e00261c33";
const DEFAULT_TX_HASH: &str = "0x0aac8b01cbcfcec9f551effbb2fd65a6378ef2193e487de97814a84a3267216e";

fn index_to_nibbles(index: u64) -> Vec<u8> {
    let mut nibbles = Vec::new();
    let mut num = index;
    
    // Convert to nibbles (4 bits each)
    while num > 0 {
        nibbles.push((num & 0xF) as u8);
        num >>= 4;
    }
    
    // Reverse to get correct order
    nibbles.reverse();
    if nibbles.is_empty() {
        nibbles.push(0);
    }
    nibbles
}

fn get_rlp_encodings(consensus_tx: &alloy::consensus::TxEnvelope) -> (Vec<u8>, Vec<u8>) {
    // Get network RLP encoding
    let network_rlp = alloy_rlp::encode(consensus_tx);
    
    // Extract EIP-2718 encoding by removing the outer RLP wrapper
    let mut network_slice = network_rlp.as_slice();
    let header = alloy_rlp::Header::decode(&mut network_slice).unwrap();
    let eip2718_bytes = network_slice[..header.payload_length].to_vec();
    
    (network_rlp, eip2718_bytes)
}

async fn get_trie_proof<P: Provider>(provider: &P, block_number: u64, tx_index: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Get the block that included the tx
    let block = provider.get_block_by_number(BlockNumberOrTag::Number(block_number)).await?.unwrap();
    
    println!("\nGenerating Transaction Trie Proof:");
    println!("Block Number: {}", block_number);
    println!("Transaction Index: {}", tx_index);
    println!("Target Nibbles: {:?}", index_to_nibbles(tx_index));
    
    // For now, let's show how the trie would be built
    println!("Total transactions in block: {}", block.transactions.len());
    
    // Show some sample proof nodes structure
    println!("\nProof Nodes Structure (conceptual):");
    println!("proof_nodes[0]: Root branch node RLP");
    println!("proof_nodes[1]: Intermediate branch/extension node RLP");
    println!("proof_nodes[2]: Leaf node containing transaction RLP");
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let provider = ProviderBuilder::new().connect(RPC_URL).await.unwrap();
    
    print!("Enter transaction hash (press Enter for default): ");
    io::stdout().flush().unwrap();
    
    let mut tx_hash_input = String::new();
    io::stdin()
        .read_line(&mut tx_hash_input)
        .expect("Failed to read transaction hash");

    // Remove trailing whitespace
    let tx_hash_input = tx_hash_input.trim();
    
    // Use default hash if input is empty
    let tx_hash_input = if tx_hash_input.is_empty() {
        println!("Using default transaction hash: {}", DEFAULT_TX_HASH);
        DEFAULT_TX_HASH
    } else {
        tx_hash_input
    };
    
    // Parse the input into B256
    let tx_hash = match B256::from_str(tx_hash_input) {
        Ok(hash) => hash,
        Err(_) => {
            println!("Invalid transaction hash format");
            return;
        }
    };
    
    println!("Original Transaction Hash: 0x{}", hex::encode(tx_hash));
    
    // Fetch transaction details
    match provider.get_transaction_by_hash(tx_hash).await {
        Ok(Some(tx)) => {
            println!("\nTransaction found!");
            let tx_index = tx.transaction_index.unwrap_or_default();
            println!("Transaction Index: {}", tx_index);
            
            // Show nibble representation
            let nibbles = index_to_nibbles(tx_index);
            println!("Transaction Index as Nibbles: {:?}", nibbles);
            println!("Nibble Path in Trie: {}", nibbles.iter()
                .map(|n| format!("{:x}", n))
                .collect::<Vec<String>>()
                .join(""));
            
            // RLP encode the index
            let rlp_encoded_index = encode(&tx_index);
            println!("RLP Encoded Index: 0x{}", hex::encode(&rlp_encoded_index));

            // Get the block number for the proof
            if let Some(block_number) = tx.block_number {
                // Get the transaction receipt to verify inclusion
                if let Ok(Some(receipt)) = provider.get_transaction_receipt(tx_hash).await {
                    println!("\nTransaction Receipt:");
                    println!("Block Number: {}", block_number);
                    println!("Transaction Index: {}", receipt.transaction_index.unwrap_or_default());
                }
                
                // Generate the trie proof
                if let Err(e) = get_trie_proof(&provider, block_number, tx_index).await {
                    println!("Error generating trie proof: {}", e);
                }
            }
            
            // Get RLP encoding via consensus conversion
            let consensus_tx: alloy::consensus::TxEnvelope = tx.try_into().unwrap();
            
            // Get both RLP encodings
            let (network_rlp, eip2718_bytes) = get_rlp_encodings(&consensus_tx);

            println!("Retrieved EIP-2718 RLP encoding: 0x{}", hex::encode(&eip2718_bytes));
        }
        Ok(None) => println!("Transaction not found"),
        Err(e) => println!("Error fetching transaction: {}", e),
    }
}
