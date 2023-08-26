use std::{fs::File, io::Read};
use rs_merkle::{MerkleTree, algorithms::Sha256, Hasher};

use super::hasher::SortingSha256Hasher;



pub fn get_merkle_tree_simple() -> MerkleTree::<SortingSha256Hasher> {

    let mut file = File::open("src/tests/data/whitelist_simple.json").unwrap();
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
