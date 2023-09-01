use std::{fs::File, io::Read};
use rs_merkle::{MerkleTree, algorithms::Sha256, Hasher};

use super::hasher::SortingSha256Hasher;


pub fn tree_from_vec(leaves: &Vec<String>) -> MerkleTree::<SortingSha256Hasher> {
    let leaves: Vec<[u8; 32]> = leaves
        .iter()
        .map(|x| Sha256::hash(x.as_bytes()))
        .collect();

    MerkleTree::<SortingSha256Hasher>::from_leaves(&leaves)
}

pub fn get_merkle_tree_simple(path_prefix: Option<String>) -> MerkleTree::<SortingSha256Hasher> {
    let path = path_prefix.unwrap_or_default() + "src/tests/data/whitelist_simple.json";
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let serialized: Vec<String> = serde_json::from_str(data.as_str()).unwrap();
    let leaves: Vec<[u8; 32]> = serialized
        .iter()
        .map(|x| Sha256::hash(x.as_bytes()))
        .collect();

    let merkle_tree = MerkleTree::<SortingSha256Hasher>::from_leaves(&leaves);
    merkle_tree
}
