use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use std::{fs::File, io::Read};

use super::hasher::SortingSha256Hasher;

fn text_from_file(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    data
}

pub fn hash_and_build_tree(serialized: &[String]) -> MerkleTree<SortingSha256Hasher> {
    let leaves: Vec<[u8; 32]> = serialized
        .iter()
        .map(|x| Sha256::hash(x.as_bytes()))
        .collect();

    MerkleTree::<SortingSha256Hasher>::from_leaves(&leaves)
}

pub fn tree_from_file(path: &str) -> MerkleTree<SortingSha256Hasher> {
    let data = text_from_file(path);

    let serialized: Vec<String> = data
        .split('\n')
        .map(|x| x.to_string())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .collect();

    hash_and_build_tree(&serialized)
}

pub fn get_merkle_tree_simple(path_prefix: Option<String>) -> MerkleTree<SortingSha256Hasher> {
    let path = path_prefix.unwrap_or_default() + "src/tests/data/whitelist_simple.txt";
    tree_from_file(path.as_str())
}

pub fn get_merkle_tree_medium() -> MerkleTree<SortingSha256Hasher> {
    let path = "src/tests/data/whitelist_medium.txt";
    tree_from_file(path)
}

pub fn get_merkle_tree_large() -> MerkleTree<SortingSha256Hasher> {
    let path = "src/tests/data/whitelist_medium.txt";
    let data = text_from_file(path);

    let mut serialized: Vec<String> = data
        .split('\n')
        .map(|x| x.to_string())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .collect();

    for _ in 0..5 {
        serialized.extend(serialized.clone());
    }

    hash_and_build_tree(&serialized)
}
