use rs_merkle::{algorithms::Sha256, Hasher};

#[derive(Clone)]
pub struct SortingSha256Hasher {}

impl Hasher for SortingSha256Hasher {
    type Hash = [u8; 32];

    fn concat_and_hash(left: &Self::Hash, right: Option<&Self::Hash>) -> Self::Hash {

        match right {
            Some(right_node) => {

                let mut both = vec![left, right_node];
                both.sort_unstable();

                let mut concatenated: Vec<u8> = (*both[0]).into();
                concatenated.append(&mut (*both[1]).into());

                Self::hash(&concatenated)
            }
            None => *left,
        }
    }

    fn hash(data: &[u8]) -> Self::Hash {
        Sha256::hash(data)
    }

    fn hash_size() -> usize {
        Sha256::hash_size()
    }
}