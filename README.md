# zk-payment

1. User submits payment verification request (tx_hash, expected_to, expected_value) 
2. Proof generation service:
   - Fetches transaction data 🟡
   - Constructs Merkle proof
   - Generates ZK proof
3. Submit proof to verification contract
4. Contract verifies proof and emits verification event
5. Return verification result to user