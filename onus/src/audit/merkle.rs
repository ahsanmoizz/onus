//! Merkle tree for tamper-evident audit trails.
//! Each session's actions are hashed into a Merkle tree; the root is stored
//! and can be externally anchored for third-party verification.

use sha2::{Digest, Sha256};

/// A simple Merkle tree built from leaf hashes.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    levels: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build a Merkle tree from leaf hashes.
    /// If there are no leaves, the root is all zeros.
    pub fn from_leaves(leaves: &[[u8; 32]]) -> Self {
        if leaves.is_empty() {
            return Self {
                levels: vec![vec![[0u8; 32]]],
            };
        }

        let mut levels = vec![leaves.to_vec()];
        let mut current = leaves.to_vec();

        // Keep hashing until we have a single root.
        // Handles the single-leaf case (creates one hash level) and the multi-level case.
        while current.len() > 1 || levels.len() == 1 {
            let mut next_level = Vec::with_capacity((current.len() + 1) / 2);

            for pair in current.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0]);

                // If there's an odd number, duplicate the last element.
                if pair.len() == 2 {
                    hasher.update(&pair[1]);
                } else {
                    hasher.update(&pair[0]);
                }

                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                next_level.push(hash);
            }

            levels.push(next_level.clone());
            current = next_level;
        }

        Self { levels }
    }

    /// Return the Merkle root hash.
    pub fn root_hash(&self) -> [u8; 32] {
        self.levels
            .last()
            .and_then(|l| l.first())
            .copied()
            .unwrap_or([0u8; 32])
    }

    /// Return the number of leaves.
    pub fn leaf_count(&self) -> usize {
        self.levels.first().map(|l| l.len()).unwrap_or(0)
    }
}

/// Compute a single SHA-256 hash for action chaining.
pub fn action_hash(
    action_id: &str,
    session_id: &str,
    sequence: u32,
    action_type: &str,
    payload: &str,
    prev_hash: Option<&str>,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(action_id.as_bytes());
    hasher.update(b"|");
    hasher.update(session_id.as_bytes());
    hasher.update(b"|");
    hasher.update(sequence.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(action_type.as_bytes());
    hasher.update(b"|");
    hasher.update(payload.as_bytes());
    if let Some(ph) = prev_hash {
        hasher.update(b"|");
        hasher.update(ph.as_bytes());
    }

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_single_leaf() {
        let leaf = [1u8; 32];
        let tree = MerkleTree::from_leaves(&[leaf]);
        assert_eq!(tree.leaf_count(), 1);
        // Single leaf: root = hash(leaf || leaf)
        let mut hasher = Sha256::new();
        hasher.update(&leaf);
        hasher.update(&leaf);
        let expected = hasher.finalize();
        let mut expected_arr = [0u8; 32];
        expected_arr.copy_from_slice(&expected);
        assert_eq!(tree.root_hash(), expected_arr);
    }

    #[test]
    fn test_merkle_two_leaves() {
        let leaf1 = [1u8; 32];
        let leaf2 = [2u8; 32];
        let tree = MerkleTree::from_leaves(&[leaf1, leaf2]);

        let mut hasher = Sha256::new();
        hasher.update(&leaf1);
        hasher.update(&leaf2);
        let expected = hasher.finalize();
        let mut expected_arr = [0u8; 32];
        expected_arr.copy_from_slice(&expected);
        assert_eq!(tree.root_hash(), expected_arr);
    }

    #[test]
    fn test_merkle_empty() {
        let tree = MerkleTree::from_leaves(&[]);
        assert_eq!(tree.root_hash(), [0u8; 32]);
        assert_eq!(tree.leaf_count(), 1); // Sentinel root
    }

    #[test]
    fn test_merkle_odd_leaves() {
        let leaves = vec![[1u8; 32], [2u8; 32], [3u8; 32]];
        let tree = MerkleTree::from_leaves(&leaves);
        // Should work without panicking (odd number duplicates last)
        assert!(tree.root_hash() != [0u8; 32]);
    }

    #[test]
    fn test_merkle_deterministic() {
        let leaves = vec![[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree1 = MerkleTree::from_leaves(&leaves);
        let tree2 = MerkleTree::from_leaves(&leaves);
        assert_eq!(tree1.root_hash(), tree2.root_hash());
    }

    #[test]
    fn test_action_hash_chaining() {
        let h1 = action_hash("id1", "session1", 1, "shell", "ls", None);
        let h2 = action_hash("id2", "session1", 2, "shell", "rm", Some(&hex::encode(h1)));

        // Different payload → different hash.
        assert_ne!(h1, h2);

        // Same inputs → same hash (with same prev_hash).
        let h2_again = action_hash("id2", "session1", 2, "shell", "rm", Some(&hex::encode(h1)));
        assert_eq!(h2, h2_again);
    }
}
