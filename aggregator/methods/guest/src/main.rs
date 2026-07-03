use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{Impl, Sha256, Digest};

fn main() {
    let n: u32 = env::read();
    let sub_id: Digest = env::read();         // image id of sub_circuit
    
    let mut leaves: Vec<[u8; 32]> = Vec::new();
    for _ in 0..n {
        let journal: Vec<u8> = env::read();
        env::verify(sub_id, &journal).expect("sub-proof invalid");  // recursive verify
        
        let hash = *Impl::hash_bytes(&journal);
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(hash.as_bytes());
        leaves.push(leaf);
    }
    
    // Simple batch root: hash(leaf1 || leaf2 || ...)
    // Note: For a proper Merkle tree, one would build a tree.
    // For MVP, just a linear hash of the leaves or just hashing the concatenated leaves.
    let mut batch_data = Vec::new();
    for leaf in leaves {
        batch_data.extend_from_slice(&leaf);
    }
    let batch_root_digest = *Impl::hash_bytes(&batch_data);
    let mut batch_root = [0u8; 32];
    batch_root.copy_from_slice(batch_root_digest.as_bytes());
    
    env::commit_slice(&batch_root);
    env::commit_slice(&n.to_le_bytes());
}
