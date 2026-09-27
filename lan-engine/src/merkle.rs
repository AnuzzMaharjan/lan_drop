use std::{fs::File, io::Read};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    hash: [u8; 32],
    chunk_index: Option<usize>,
}

impl Node {
    fn new(
        left: Option<Box<Node>>,
        right: Option<Box<Node>>,
        hash: [u8; 32],
        chunk_index: Option<usize>,
    ) -> Self {
        Node {
            left,
            right,
            hash,
            chunk_index,
        }
    }
    fn hash_data(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}

#[derive(Debug)]
pub struct MerkleTree {
    root: Option<Node>,
}

impl MerkleTree {
    pub fn new(file_path: &str) -> Result<(Self, Vec<[u8; 32]>), std::io::Error> {
        let mut file = File::open(file_path)?;
        let mut leaf_hashes = Vec::new();
        // let mut buffer = [0u8; 8192];
        let mut buffer = [0u8; 8192];

        while let Ok(bytes_read) = file.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }

            let hash = Node::hash_data(&buffer[..bytes_read]);
            leaf_hashes.push(hash);
        }

        let tree = MerkleTree::build_from_hashes(leaf_hashes.clone());

        Ok((tree, leaf_hashes))
    }

    pub fn build_from_hashes(hashes: Vec<[u8; 32]>) -> Self {
        let mut current_level: Vec<Node> = hashes
            .into_iter()
            .enumerate()
            .map(|(index, hash)| Node::new(None, None, hash, Some(index)))
            .collect();

        while current_level.len() > 1 {
            if current_level.len() % 2 == 1 {
                current_level.push(Node::new(None, None, [0u8; 32], Some(current_level.len())));
            }

            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let left = chunk[0].clone();
                let right = chunk[1].clone();
                let combined_hash = Node::hash_nodes(&left.hash, &right.hash);

                next_level.push(Node::new(
                    Some(Box::new(left)),
                    Some(Box::new(right)),
                    combined_hash,
                    None,
                ));
            }
            current_level = next_level;
        }

        MerkleTree {
            root: current_level.into_iter().next(),
        }
    }
}

impl MerkleTree {
    pub fn get_root_hash(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|n| n.hash)
    }

    pub fn root_hex(&self) -> String {
        self.root
            .as_ref()
            .map(|n| hex::encode(n.hash))
            .unwrap_or_default()
    }

    pub fn verify_root(&self, other_root: [u8; 32]) -> bool {
        self.get_root_hash() == Some(other_root)
    }
}