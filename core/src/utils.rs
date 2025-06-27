/// Convert an index to nibbles (4-bit values)
pub fn index_to_nibbles(index: u64) -> Vec<u8> {
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

/// Format nibbles as a hex string path
pub fn format_nibble_path(nibbles: &[u8]) -> String {
    nibbles.iter()
        .map(|n| format!("{:x}", n))
        .collect::<Vec<String>>()
        .join("")
} 