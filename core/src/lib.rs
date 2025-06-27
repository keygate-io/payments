pub mod transaction;
pub mod proof;
pub mod utils;
pub mod rpc;

// Re-export main functionality for easy access
pub use transaction::{get_rlp_encodings, write_rlp_to_file, process_transaction_rlp};
pub use proof::get_proof_nodes;
pub use utils::{index_to_nibbles, format_nibble_path};
pub use rpc::{connect_provider, fetch_transaction_details, process_full_transaction}; 