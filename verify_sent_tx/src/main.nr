mod constants;
mod rlp;

//     tx_len: u32,
// proof_path: [u8; constants::PATH_LEN],
// proof_nodes: [Field; constants::N_NODES],

// // Public inputs
// block_tx_root: Field,
// expected_to: Field,
// expected_value: Field

fn main(
    // Private inputs
    tx_rlp: [u8; constants::TX_RLP_MAX],
) {
    let tx = rlp::decode_to_eip1559_transaction(tx_rlp);
    
    // Print transaction details
    std::println("Transaction details");
    std::println(tx);
    assert(true);
}
