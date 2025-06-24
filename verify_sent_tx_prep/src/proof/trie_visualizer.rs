use alloy_trie::Nibbles;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TrieNode {
    pub nibbles: Nibbles,
    pub is_leaf: bool,
    pub children: HashMap<u8, TrieNode>,
    pub value: Option<Vec<u8>>,
}

impl TrieNode {
    pub fn new(nibbles: Nibbles) -> Self {
        Self {
            nibbles,
            is_leaf: false,
            children: HashMap::new(),
            value: None,
        }
    }

    pub fn add_leaf(&mut self, nibbles: Nibbles, value: Vec<u8>) {
        self.insert_path(nibbles, value, 0);
    }

    fn insert_path(&mut self, nibbles: Nibbles, value: Vec<u8>, depth: usize) {
        if depth >= nibbles.len() {
            self.is_leaf = true;
            self.value = Some(value);
            return;
        }

        let current_nibble = nibbles.get(depth).unwrap_or(&0);
        let child = self.children.entry(current_nibble.clone()).or_insert_with(|| {
        let mut child_nibbles = self.nibbles.clone();
            child_nibbles.push(current_nibble.clone());
            TrieNode::new(child_nibbles)
        });

        child.insert_path(nibbles, value, depth + 1);
    }

    pub fn print_tree(&self, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let next_prefix = if is_last { "    " } else { "│   " };
        
        let nibble_str = format!("{:?}", self.nibbles);
        let leaf_indicator = if self.is_leaf { " (LEAF)" } else { "" };
        
        println!("{}{}{}{}", prefix, connector, nibble_str, leaf_indicator);
        
        let mut children: Vec<_> = self.children.iter().collect();
        children.sort_by(|a, b| a.0.cmp(b.0));
        
        for (i, (_, child)) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            child.print_tree(&format!("{}{}", prefix, next_prefix), is_last_child);
        }
    }

    pub fn find_proof_path(&self, target_nibbles: &Nibbles) -> Vec<Nibbles> {
        let mut proof_path = Vec::new();
        self.collect_proof_nodes(target_nibbles, &mut proof_path);
        proof_path
    }

    fn collect_proof_nodes(&self, target_nibbles: &Nibbles, proof_path: &mut Vec<Nibbles>) {
        // Add current node to proof if it's on the path to target
        if self.is_on_path_to(target_nibbles) {
            proof_path.push(self.nibbles.clone());
        }

        // Recursively collect from children
        for child in self.children.values() {
            child.collect_proof_nodes(target_nibbles, proof_path);
        }
    }

    fn is_on_path_to(&self, target_nibbles: &Nibbles) -> bool {
        // Check if this node's nibbles are a prefix of the target
        if self.nibbles.len() > target_nibbles.len() {
            return false;
        }
        
        for (i, &nibble) in self.nibbles.iter().enumerate() {
            if i >= target_nibbles.len() || target_nibbles.get(i).unwrap_or(&0).clone() != nibble {
                return false;
            }
        }
        true
    }

    pub fn get_statistics(&self) -> TrieStats {
        let mut stats = TrieStats::default();
        self.collect_stats(&mut stats);
        stats
    }

    fn collect_stats(&self, stats: &mut TrieStats) {
        stats.total_nodes += 1;
        if self.is_leaf {
            stats.leaf_nodes += 1;
        }
        stats.max_depth = stats.max_depth.max(self.nibbles.len());
        
        for child in self.children.values() {
            child.collect_stats(stats);
        }
    }
}

#[derive(Debug, Default)]
pub struct TrieStats {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub max_depth: usize,
}

pub fn build_trie_from_transactions(transactions: &[(Nibbles, Vec<u8>)]) -> TrieNode {
    let mut root = TrieNode::new(Nibbles::default());
    
    for (nibbles, value) in transactions {
        root.add_leaf(nibbles.clone(), value.clone());
    }
    
    root
}

pub fn visualize_trie(transactions: &[(Nibbles, Vec<u8>)], target_nibbles: Option<&Nibbles>) {
    println!("=== TRIE VISUALIZATION ===");
    println!("Total transactions: {}", transactions.len());
    
    let trie = build_trie_from_transactions(transactions);
    let stats = trie.get_statistics();
    
    println!("\nTrie Statistics:");
    println!("  Total nodes: {}", stats.total_nodes);
    println!("  Leaf nodes: {}", stats.leaf_nodes);
    println!("  Max depth: {}", stats.max_depth);
    
    println!("\nTrie Structure:");
    trie.print_tree("", true);
    
    if let Some(target) = target_nibbles {
        println!("\nProof path for target nibbles {:?}:", target);
        let proof_path = trie.find_proof_path(target);
        for (i, nibbles) in proof_path.iter().enumerate() {
            println!("  {}: {:?}", i, nibbles);
        }
    }
    
    println!("\n=== END VISUALIZATION ===");
} 