mod constants;
mod rlp;

// tx_len: u32,
// proof_path: [u8; constants::PATH_LEN],
// proof_nodes: [Field; constants::N_NODES],

// // Public inputs
// block_tx_root: Field,

fn main(
    // Private inputs
    tx_rlp: [u8; constants::TX_RLP_MAX],
    expected_to: Field,
    expected_value: Field,
) {
    let tx = rlp::decode_to_eip1559_transaction(tx_rlp);

    // Assert that the transaction is sent to expected_to
    assert(tx.to == expected_to);

    // Assert that the transaction is sent with expected_value
    assert(tx.value == expected_value);

    // Print transaction details
    assert(true);
}
