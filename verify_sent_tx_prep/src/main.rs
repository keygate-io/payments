use std::env;
use serde_json::{json};

// Import our modules
mod transaction;
mod proof;
mod utils;
mod rpc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        let error_result = json!({
            "success": false,
            "error": "Usage: keygate <command> [options]\n\nCommands:\n  prepare <transaction_hash> <expected_to> <expected_value> [--debug]\n  rlp-of <transaction_hash> [--debug]\n  proof <transaction_hash> [--debug]\n  visualize-nibble-trie <transaction_hash> [--debug]"
        });
        println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
        std::process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "prepare" => {
            if args.len() < 5 {
                let error_result = json!({
                    "success": false,
                    "error": "Usage: keygate prepare <transaction_hash> <expected_to> <expected_value> [--debug]"
                });
                println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                std::process::exit(1);
            }
            
            let tx_hash = &args[2];
            let expected_to = &args[3];
            let expected_value = &args[4];
            let debug = args.contains(&String::from("--debug"));
            
            let result = rpc::process_full_transaction(tx_hash, expected_to, expected_value).await;
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        "rlp-of" => {
            if args.len() < 3 {
                let error_result = json!({
                    "success": false,
                    "error": "Usage: keygate rlp-of <transaction_hash> [--debug]"
                });
                println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                std::process::exit(1);
            }
            
            let tx_hash_input = &args[2];
            let debug = args.contains(&String::from("--debug"));
            
            // Fetch transaction and process RLP encodings
            match rpc::fetch_transaction_details(tx_hash_input).await {
                Ok(tx) => {
                    match transaction::process_transaction_rlp(&tx) {
                        Ok(result) => {
                            if debug {
                                if let Some(rlp_encodings) = result.get("rlp_encodings") {
                                    if let Some(network_rlp) = rlp_encodings.get("network_rlp_hex") {
                                        eprintln!("Debug: Network RLP: {}", network_rlp.as_str().unwrap_or(""));
                                    }
                                    if let Some(eip2718_bytes) = rlp_encodings.get("eip2718_rlp_bytes") {
                                        if let Some(bytes_array) = eip2718_bytes.as_array() {
                                            eprintln!("Debug: EIP-2718 bytes length: {} bytes", bytes_array.len());
                                        }
                                    }
                                }
                            }
                            
                            println!("{}", serde_json::to_string_pretty(&result).unwrap());
                        }
                        Err(e) => {
                            let error_result = json!({
                                "success": false,
                                "error": e
                            });
                            println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    let error_result = json!({
                        "success": false,
                        "error": e
                    });
                    println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                    std::process::exit(1);
                }
            }
        }
        "proof" => {
            if args.len() < 3 {
                let error_result = json!({
                    "success": false,
                    "error": "Usage: keygate proof <transaction_hash> [--debug]"
                });
                println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                std::process::exit(1);
            }
            
            let tx_hash = &args[2];
            let debug = args.contains(&String::from("--debug"));
            
            // Get transaction details to find block number
            match rpc::fetch_transaction_details(tx_hash).await {
                Ok(tx) => {
                    if let Some(block_number) = tx.block_number {
                        match proof::get_proof_nodes(block_number, tx_hash).await {
                            Ok(result) => {
                                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                            }
                            Err(e) => {
                                let error_result = json!({
                                    "success": false,
                                    "error": format!("Failed to generate proof: {}", e)
                                });
                                println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                                std::process::exit(1);
                            }
                        }
                    } else {
                        let error_result = json!({
                            "success": false,
                            "error": "Transaction not yet included in a block"
                        });
                        println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let error_result = json!({
                        "success": false,
                        "error": format!("Failed to fetch transaction: {}", e)
                    });
                    println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                    std::process::exit(1);
                }
            }
        }
        "visualize-nibble-trie" => {
            if args.len() < 3 {
                let error_result = json!({
                    "success": false,
                    "error": "Usage: keygate visualize-nibble-trie <transaction_hash> [--debug]"
                });
                println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                std::process::exit(1);
            }
            
            let tx_hash = &args[2];
            let debug = args.contains(&String::from("--debug"));
            
            // Get transaction details to find block number
            match rpc::fetch_transaction_details(tx_hash).await {
                Ok(tx) => {
                    if let Some(block_number) = tx.block_number {
                        match proof::visualize_trie_for_transaction(block_number, tx_hash).await {
                            Ok(_) => {
                                // Visualization is printed directly, no JSON output needed
                            }
                            Err(e) => {
                                let error_result = json!({
                                    "success": false,
                                    "error": format!("Failed to visualize trie: {}", e)
                                });
                                println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                                std::process::exit(1);
                            }
                        }
                    } else {
                        let error_result = json!({
                            "success": false,
                            "error": "Transaction not yet included in a block"
                        });
                        println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let error_result = json!({
                        "success": false,
                        "error": format!("Failed to fetch transaction: {}", e)
                    });
                    println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
                    std::process::exit(1);
                }
            }
        }
        _ => {
            let error_result = json!({
                "success": false,
                "error": format!("Unknown command: {}\n\nUsage: keygate <command> [options]\n\nCommands:\n  prepare <transaction_hash> <expected_to> <expected_value> [--debug]\n  rlp-of <transaction_hash> [--debug]\n  proof <transaction_hash> [--debug]\n  visualize-nibble-trie <transaction_hash> [--debug]", command)
            });
            println!("{}", serde_json::to_string_pretty(&error_result).unwrap());
            std::process::exit(1);
        }
    }
}
