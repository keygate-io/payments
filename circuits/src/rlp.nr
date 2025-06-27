use crate::constants;
use std::convert::AsPrimitive;

pub struct TxEip1559 {
    pub chain_id: Field,
    pub nonce: Field,
    pub max_priority_fee: Field,
    pub max_fee: Field,
    pub gas_limit: Field,
    pub to: Field,
    pub value: Field,
    pub y_parity: Field,
    pub r: Field,
    pub s: Field,
}

// TODO: Optimize constants::TX_RLP_MAX size
pub fn list_size_byte_to_len(raw: [u8; constants::TX_RLP_MAX], cursor: u8) -> (Field, Field) {
    let byte_list_size = raw[cursor];
    // Short list
    if (byte_list_size >= 0xC0) {
        ((byte_list_size - 0xC0) as Field, (cursor + 1) as Field)
    } else if (byte_list_size >= 0xF7) {
        let bytes_of_len = byte_list_size - 0xF7;
        let mut len: Field = 0;
        // Use a fixed-size loop with maximum possible length
        for i in 0..8 {
            if (i < bytes_of_len) {
                len = 256 * len + (raw[cursor + 1 + i] as Field);
            }
        }
        (len, (cursor + bytes_of_len + 1) as Field)
    } else {
        (0, cursor as Field)
    }
}

pub fn scalar_rlp_to_field(raw: [u8; constants::TX_RLP_MAX], cursor: u8) -> (Field, Field) {
    let byte_size = raw[cursor];
    if (byte_size <= 0x7F) {
        (byte_size as Field, (cursor + 1) as Field)
    } else if (byte_size <= 0xB7) {
        let len = byte_size - 0x80;
        let mut value: Field = 0;
        // Use a fixed-size loop with maximum possible length
        for i in 0..32 {
            if (i < len) {
                value = 256 * value + (raw[cursor + 1 + i] as Field);
            }
        }
        (value, (cursor + len + 1) as Field)
    } else {
        (0, cursor as Field)
    }
}

pub fn string_size_byte_to_len(raw: [u8; constants::TX_RLP_MAX], cursor: u8) -> (Field, Field) {
    let byte_size = raw[cursor];
    // Short string
    if (byte_size <= 0xB7) {
        ((byte_size - 0x80) as Field, (cursor + 1) as Field)
    } else if (byte_size <= 0xBF) {
        let bytes_of_len = byte_size - 0xB7;
        let mut len: Field = 0;
        // Use a fixed-size loop with maximum possible length
        for i in 0..8 {
            if (i < bytes_of_len) {
                len = 256 * len + (raw[cursor + 1 + i] as Field);
            }
        }
        (len, (cursor + bytes_of_len + 1) as Field)
    } else {
        (0, cursor as Field)
    }
}

pub fn decode_to_eip1559_rlp_to_field(raw: [u8; constants::TX_RLP_MAX], cursor: u8) -> (Field, u8) {
    // This function is not used, returning default values
    (0, cursor)
}

// Universal RLP field parser that handles all encoding types
pub fn parse_rlp_field(raw: [u8; constants::TX_RLP_MAX], cursor: u32) -> (Field, u32) {
    let first_byte = raw[cursor];
    
    if (first_byte <= 0x7f) {
        // CASE 1: Single byte value (0x00 - 0x7f)
        // The byte itself is the value
        (first_byte as Field, cursor + 1)
        
    } else if (first_byte <= 0xb7) {
        // CASE 2: Short string (0x80 - 0xb7)
        // Length = first_byte - 0x80
        let length = first_byte - 0x80;
        if (length == 0) {
            (0, cursor + 1) // Empty string
        } else {
            // Read 'length' bytes and convert to Field
            let mut value: Field = 0;
            for i in 0..32 { // Fixed loop for Noir
                if ((i as u8) < length) {
                    value = value * 256 + (raw[cursor + 1 + i] as Field);
                }
            }
            (value, cursor + 1 + (length as u32))
        }
        
    } else if (first_byte <= 0xbf) {
        // CASE 3: Long string (0xb8 - 0xbf)
        // Length of length = first_byte - 0xb7
        let length_of_length = first_byte - 0xb7;
        
        // Read the length
        let mut actual_length: u32 = 0;
        for i in 0..8 { // Fixed loop
            if ((i as u8) < length_of_length) {
                actual_length = actual_length * 256 + (raw[cursor + 1 + i] as u32);
            }
        }
        
        // Read the actual data
        let mut value: Field = 0;
        for i in 0..32 { // Fixed loop
            if (i < actual_length) {
                value = value * 256 + (raw[cursor + 1 + (length_of_length as u32) + i] as Field);
            }
        }
        (value, cursor + 1 + (length_of_length as u32) + actual_length)
        
    } else {
        // CASE 4: Lists (0xc0 - 0xff) - for empty lists, return 0
        if (first_byte == 0xc0) {
            (0, cursor + 1) // Empty list
        } else if (first_byte <= 0xf7) {
            // Short list - skip over it
            let list_length = first_byte - 0xc0;
            (0, cursor + 1 + (list_length as u32))
        } else {
            // Long list - skip over it
            let length_of_length = first_byte - 0xf7;
            let mut actual_length: u32 = 0;
            for i in 0..8 {
                if ((i as u8) < length_of_length) {
                    actual_length = actual_length * 256 + (raw[cursor + 1 + i] as u32);
                }
            }
            (0, cursor + 1 + (length_of_length as u32) + actual_length)
        }
    }
}

// Special parser for addresses (always 20 bytes)
pub fn parse_address_field(raw: [u8; constants::TX_RLP_MAX], cursor: u32) -> (Field, u32) {
    let first_byte = raw[cursor];
    assert(first_byte == 0x94, "Expected address field (0x94 + 20 bytes)");
    
    // Read 20 bytes and convert to Field (only first 20 bytes fit in Field)
    let mut value: Field = 0;
    for i in 0..20 {
        value = value * 256 + (raw[cursor + 1 + i] as Field);
    }
    (value, cursor + 21)
}

// Special parser for signatures (always 32 bytes)
pub fn parse_signature_field(raw: [u8; constants::TX_RLP_MAX], cursor: u32) -> (Field, u32) {
    let first_byte = raw[cursor];
    assert(first_byte == 0xa0, "Expected signature field (0xa0 + 32 bytes)");
    
    // Read 32 bytes and convert to Field
    let mut value: Field = 0;
    for i in 0..32 {
        value = value * 256 + (raw[cursor + 1 + i] as Field);
    }
    (value, cursor + 33)
}

/**
    Transactions are represented as RLP-encoded byte lists.
    [HEADER] The beginning of a RLP encoding is the transaction type byte -- 0x02 for EIP-1559 transactions.
    [HEADER] Then, the next byte details the size of the list. [short = 0x80 .. 0xB7, long = 0xB7 .. 0xBF]
    [HEADER] The third byte is the transaction payload length that follows (ignoring the header bytes).
    [PAYLOAD] We have n bytes of payload, and we need to decode them into a transaction.
**/
pub fn decode_to_eip1559_transaction(raw: [u8; constants::TX_RLP_MAX]) -> TxEip1559 {
    // [HEADER] Validate transaction type
    let tx_type = raw[0];
    assert(tx_type == 0x02, "Invalid transaction type. Expected 0x02");
    assert(raw[1] > 0xC0, "Invalid RLP format. Expected list");

    // Skip header: type (1 byte) + list header (2 bytes for 0xf8 0x70).
    let mut cursor: u32 = 3;
    
    // Parse each field in order using the universal parser
    let (chain_id, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    let (nonce, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    let (max_priority_fee, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    let (max_fee, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    let (gas_limit, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;

    // Special handling for address (always 20 bytes)
    let (to_address, new_cursor) = parse_address_field(raw, cursor);
    cursor = new_cursor;
    
    let (value, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    // Data field (usually empty)
    let (data_length, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    // Access list (usually empty)
    let (access_list, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    // Y parity
    let (y_parity, new_cursor) = parse_rlp_field(raw, cursor);
    cursor = new_cursor;
    
    // R signature (32 bytes)
    let (r_sig, new_cursor) = parse_signature_field(raw, cursor);
    cursor = new_cursor;
    
    // S signature (32 bytes)
    let (s_sig, new_cursor) = parse_signature_field(raw, cursor);
    cursor = new_cursor;

    TxEip1559 {
        chain_id: chain_id,
        nonce: nonce,
        max_priority_fee: max_priority_fee,
        max_fee: max_fee,
        gas_limit: gas_limit,
        to: to_address,
        value: value,
        y_parity: y_parity,
        r: r_sig,
        s: s_sig,
    }
}
