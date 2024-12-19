use cosmwasm_schema::cw_serde;

use cosmwasm_std::{StdError, Storage};
use cw_storage_plus::Item;
use nois::{int_in_range, shuffle as nois_shuffle, sub_randomness_with_key};
use thiserror::Error;
// BUCKET_SIZE is limited to 256 to efficiently store ids as u8 partitioning into multiple buckets
pub const BUCKET_SIZE: u32 = 256;
pub const MAX_BUCKETS: u32 = 256;
pub const MAX_SIZE: u32 = MAX_BUCKETS * BUCKET_SIZE;

// buckets returns the number of necessary buckets for a given collection size.
// it returns the number of buckets and the size of the last bucket
fn buckets(size: u32) -> (u32, u32) {
    let buckets = size / BUCKET_SIZE;
    let remainder = size % BUCKET_SIZE;
    if remainder > 0 {
        return (buckets + 1, remainder);
    }
    (buckets, BUCKET_SIZE)
}

// bucket_key returns the key for a given bucket id
pub fn bucket_key(bucket_id: u8) -> [u8; 4] {
    [
        0xAA, // prefix
        b'v', // vending
        b'm', // minter
        bucket_id,
    ]
}

const AVAILABLE_BUCKETS_KEY: [u8; 3] = [0xAA, b'a', b'b'];

pub const COUNTERS: Item<Counters> = Item::new("counters");

#[cw_serde]
pub struct Counters {
    pub available_buckets: u32,
    pub available_items: u32,
}

#[derive(Error, Debug, PartialEq)]
pub enum MinterUtilsError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Invalid collection size {size}. Max size is {max}")]
    InvalidSize { size: u32, max: u32 },

    #[error("No available buckets")]
    NoAvailableBuckets {},

    #[error("Invalid bucket {bucket_id}")]
    InvalidBucket { bucket_id: u32 },

    #[error("Invalid token id {token_id}")]
    InvalidTokenId { token_id: u32 },
}

pub fn initialize_and_shuffle(
    storage: &mut dyn Storage,
    size: u32,
    random_seed: [u8; 32],
) -> Result<(), MinterUtilsError> {
    initialize(storage, size)?;
    shuffle(storage, random_seed)?;
    Ok(())
}

pub fn initialize(storage: &mut dyn Storage, size: u32) -> Result<(), MinterUtilsError> {
    if size > MAX_SIZE {
        return Err(MinterUtilsError::InvalidSize {
            size,
            max: MAX_SIZE,
        });
    }
    let (buckets, last_bucket_size) = buckets(size);
    for bucket in 0..buckets {
        let key = bucket_key(bucket as u8);
        let size = if bucket == buckets - 1 {
            last_bucket_size
        } else {
            BUCKET_SIZE
        };
        let bucket_contents: Vec<u8> = (0..size).map(|x| x as u8).collect();
        storage.set(&key, &bucket_contents);
    }
    let available_buckets: Vec<u8> = (0..buckets).map(|x| x as u8).collect();
    storage.set(&AVAILABLE_BUCKETS_KEY, &available_buckets);

    let counters = Counters {
        available_buckets: buckets,
        available_items: size,
    };
    COUNTERS.save(storage, &counters)?;
    Ok(())
}

fn get_bucket_and_index(token_id: u32) -> (u32, u32) {
    let bucket_id = (token_id - 1) / BUCKET_SIZE;
    let index = (token_id - 1) % BUCKET_SIZE;
    (bucket_id, index)
}

fn get_token_id(bucket_id: u32, index: u32) -> u32 {
    bucket_id * BUCKET_SIZE + index + 1
}

pub fn pick_any(storage: &mut dyn Storage, seed: [u8; 32]) -> Result<u32, MinterUtilsError> {
    let Some(mut available_buckets) = storage.get(&AVAILABLE_BUCKETS_KEY) else {
        return Err(MinterUtilsError::NoAvailableBuckets {});
    };
    let mut provider = sub_randomness_with_key(seed, b"pick_any");
    let bucket_index = int_in_range(provider.provide(), 0, available_buckets.len() - 1) as u32;
    let bucket_id = available_buckets[bucket_index as usize];
    let bucket_key = bucket_key(bucket_id);
    let Some(mut bucket) = storage.get(&bucket_key) else {
        return Err(MinterUtilsError::InvalidBucket {
            bucket_id: bucket_id as u32,
        });
    };

    let index = int_in_range(provider.provide(), 0, bucket.len() - 1) as u32;

    // available correlative
    let correlative = bucket[index as usize];
    let token_id = get_token_id(bucket_id as u32, correlative as u32);
    // item has been picked, remove it from the bucket
    bucket.remove(index as usize);

    // if the bucket is empty, remove it from the available buckets
    if bucket.is_empty() {
        available_buckets.remove(bucket_index as usize);
        // if there are no more buckets, remove the available buckets key else update it with the remaining buckets
        if available_buckets.is_empty() {
            storage.remove(&AVAILABLE_BUCKETS_KEY);
        } else {
            storage.set(&AVAILABLE_BUCKETS_KEY, &available_buckets);
        }
        storage.remove(&bucket_key);
    } else {
        storage.set(&bucket_key, &bucket);
    }
    Ok(token_id)
}

pub fn purge_buckets(storage: &mut dyn Storage, max_buckets: u32) -> Result<u32, MinterUtilsError> {
    let Some(mut available_buckets) = storage.get(&AVAILABLE_BUCKETS_KEY) else {
        return Err(MinterUtilsError::NoAvailableBuckets {});
    };
    let mut buckets_to_remove = 0;
    for i in 0..available_buckets.len() {
        if i as u32 >= max_buckets {
            break;
        }
        let bucket_key = bucket_key(available_buckets[i]);
        storage.remove(&bucket_key);
        buckets_to_remove += 1;
    }

    // remove from the end of the vector
    available_buckets.truncate(available_buckets.len() - buckets_to_remove);
    if available_buckets.is_empty() {
        storage.remove(&AVAILABLE_BUCKETS_KEY);
    } else {
        storage.set(&AVAILABLE_BUCKETS_KEY, &available_buckets);
    }
    Ok(buckets_to_remove as u32)
}

pub fn shuffle(storage: &mut dyn Storage, random_seed: [u8; 32]) -> Result<(), MinterUtilsError> {
    let mut provider = sub_randomness_with_key(random_seed, b"shuffle");
    let Some(available_buckets) = storage.get(&AVAILABLE_BUCKETS_KEY) else {
        return Err(MinterUtilsError::NoAvailableBuckets {});
    };
    let shuffled = nois_shuffle(provider.provide(), available_buckets);
    storage.set(&AVAILABLE_BUCKETS_KEY, &shuffled);
    Ok(())
}

pub fn pick_token(storage: &mut dyn Storage, token_id: u32) -> Result<u32, MinterUtilsError> {
    let Some(mut available_buckets) = storage.get(&AVAILABLE_BUCKETS_KEY) else {
        return Err(MinterUtilsError::NoAvailableBuckets {});
    };
    let (bucket_id, correlative) = get_bucket_and_index(token_id);

    let bucket_id = bucket_id as u8;
    // bucket ids can be random
    let Some(bucket_index) = available_buckets.iter().position(|item| *item == bucket_id) else {
        return Err(MinterUtilsError::InvalidBucket {
            bucket_id: bucket_id as u32,
        });
    };

    let bucket_key = bucket_key(bucket_id);
    let Some(mut bucket) = storage.get(&bucket_key) else {
        return Err(MinterUtilsError::InvalidBucket {
            bucket_id: bucket_id as u32,
        });
    };
    let correlative = correlative as u8;
    // items within a bucket are sorted
    let Ok(token_index) = bucket.binary_search(&correlative) else {
        return Err(MinterUtilsError::InvalidTokenId { token_id });
    };

    // item has been picked, remove it from the bucket
    bucket.remove(token_index);

    // if the bucket is empty, remove it from the available buckets
    if bucket.is_empty() {
        available_buckets.remove(bucket_index);
        // if there are no more buckets, remove the available buckets key else update it with the remaining buckets
        if available_buckets.is_empty() {
            storage.remove(&AVAILABLE_BUCKETS_KEY);
        } else {
            dbg!("Removing bucket {:?} {:?}", bucket_id, &available_buckets);
            storage.set(&AVAILABLE_BUCKETS_KEY, &available_buckets);
        }
        storage.remove(&bucket_key);
    } else {
        storage.set(&bucket_key, &bucket);
    }

    Ok(token_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use nois::sub_randomness_with_key;

    #[test]
    fn test_initialize() {
        let mut deps = mock_dependencies();
        initialize(&mut deps.storage, 256).unwrap();
        let counters = COUNTERS.load(&deps.storage).unwrap();
        assert_eq!(counters.available_buckets, 1);
        assert_eq!(counters.available_items, 256);

        initialize(&mut deps.storage, 10000).unwrap();
        let counters = COUNTERS.load(&deps.storage).unwrap();
        assert_eq!(counters.available_buckets, 40);
        assert_eq!(counters.available_items, 10000);

        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY).unwrap();
        assert_eq!(available_buckets.len(), 40);
        assert_eq!(
            available_buckets,
            (0..40).map(|x| x as u8).collect::<Vec<u8>>()
        );
        for bucket in 0..40 {
            let key = bucket_key(bucket as u8);
            let bucket_contents = deps.storage.get(&key).unwrap();
            if bucket == 39 {
                assert_eq!(bucket_contents.len(), 10000 % 256);
                assert_eq!(
                    bucket_contents,
                    (0..10000 % 256).map(|x| x as u8).collect::<Vec<u8>>()
                );
            } else {
                assert_eq!(bucket_contents.len(), 256);
                assert_eq!(
                    bucket_contents,
                    (0..256).map(|x| x as u8).collect::<Vec<u8>>()
                );
            }
        }
    }

    #[test]
    fn test_find_bucket_and_correlative() {
        let (n_buckets, last_bucket_size) = buckets(1);
        assert_eq!(n_buckets, 1);
        assert_eq!(last_bucket_size, 1);

        let (n_buckets, last_bucket_size) = buckets(BUCKET_SIZE);
        assert_eq!(n_buckets, 1);
        assert_eq!(last_bucket_size, BUCKET_SIZE);

        let (n_buckets, last_bucket_size) = buckets(BUCKET_SIZE + 1);
        assert_eq!(n_buckets, 2);
        assert_eq!(last_bucket_size, 1);

        let (n_buckets, last_bucket_size) = buckets(BUCKET_SIZE * 2);
        assert_eq!(n_buckets, 2);
        assert_eq!(last_bucket_size, BUCKET_SIZE);

        let (n_buckets, last_bucket_size) = buckets(BUCKET_SIZE * 2 + 1);
        assert_eq!(n_buckets, 3);
        assert_eq!(last_bucket_size, 1);

        let (n_buckets, last_bucket_size) = buckets(10_000);
        // there is 40 buckets  but the last one hast only 55 tokens
        assert_eq!(n_buckets, 40);
        assert_eq!(last_bucket_size, 16);
    }
    #[test]
    fn test_generate_mintable_tokens() {
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY).unwrap();
        assert_eq!(available_buckets.len(), 40);
        let bucket = deps.storage.get(&bucket_key(0)).unwrap();
        assert_eq!(bucket.len(), BUCKET_SIZE as usize);
        assert_eq!(
            bucket,
            (0..BUCKET_SIZE).map(|x| x as u8).collect::<Vec<u8>>()
        );
        let bucket = deps.storage.get(&bucket_key(39)).unwrap();
        assert_eq!(bucket.len(), 16);
        assert_eq!(bucket, (0..16).map(|x| x as u8).collect::<Vec<u8>>());
    }

    #[test]
    fn test_get_bucket_and_index() {
        let (bucket_id, index) = get_bucket_and_index(1);
        assert_eq!(bucket_id, 0);
        assert_eq!(index, 0);
        let token_id = get_token_id(bucket_id, index);
        assert_eq!(token_id, 1);

        let (bucket_id, index) = get_bucket_and_index(256);
        assert_eq!(bucket_id, 0);
        assert_eq!(index, 255);
        let token_id = get_token_id(bucket_id, index);
        assert_eq!(token_id, 256);

        let (bucket_id, index) = get_bucket_and_index(257);
        assert_eq!(bucket_id, 1);
        assert_eq!(index, 0);
        let token_id = get_token_id(bucket_id, index);
        assert_eq!(token_id, 257);

        let (bucket_id, index) = get_bucket_and_index(10_000);
        assert_eq!(bucket_id, 39);
        assert_eq!(index, 15);
        let token_id = get_token_id(bucket_id, index);
        assert_eq!(token_id, 10_000);

        let (bucket_id, index) = get_bucket_and_index(5_000);
        assert_eq!(bucket_id, 19);
        assert_eq!(index, 135);
        let token_id = get_token_id(bucket_id, index);
        assert_eq!(token_id, 5_000);
    }

    #[test]
    fn test_pick_any() {
        let mut provider = sub_randomness_with_key([0; 32], b"token_generator");
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let token_id = pick_any(&mut deps.storage, provider.provide()).unwrap();
        assert_eq!(token_id, 975);

        let (bucket_id, _) = get_bucket_and_index(token_id);
        let bucket = deps.storage.get(&bucket_key(bucket_id as u8)).unwrap();
        assert_eq!(bucket.len(), BUCKET_SIZE as usize - 1);
        let token_id = pick_any(&mut deps.storage, provider.provide()).unwrap();
        assert_eq!(token_id, 367);
        let (bucket_id, _) = get_bucket_and_index(token_id);
        let bucket = deps.storage.get(&bucket_key(bucket_id as u8)).unwrap();
        assert_eq!(bucket.len(), BUCKET_SIZE as usize - 1);
    }
    #[test]
    fn test_pick_all() {
        let mut provider = sub_randomness_with_key([0; 32], b"token_generator");
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 1000);
        assert!(r.is_ok());
        let mut picked = vec![];
        for _ in 0..1000 {
            let result = pick_any(&mut deps.storage, provider.provide());

            if result.is_err() {
                dbg!("Error: {:?}", &result);
            }
            assert!(result.is_ok());
            picked.push(result.unwrap());
        }
        let r = pick_any(&mut deps.storage, provider.provide());
        assert!(r.is_err());
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY);
        picked.sort();
        picked.dedup();
        assert_eq!(picked.len(), 1000);
        assert!(available_buckets.is_none());
    }

    #[test]
    fn test_pick_all_with_pick_token() {
        let mut provider = sub_randomness_with_key([0; 32], b"token_generator");
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 1000);
        assert!(r.is_ok());
        let mut picked = vec![];
        let to_be_picked = vec![1, 257, 500, 750, 1000];
        for token_id in to_be_picked {
            let result = pick_token(&mut deps.storage, token_id);
            assert!(result.is_ok());
            picked.push(result.unwrap());
        }
        for idx in 0..1000 {
            let result = pick_any(&mut deps.storage, provider.provide());
            // we should not be able to pick more than 995 tokens because we already picked 5
            if idx >= 995 {
                assert!(result.is_err());
            } else {
                assert!(result.is_ok());
                picked.push(result.unwrap());
            }
        }
        let r = pick_any(&mut deps.storage, provider.provide());
        assert!(r.is_err());
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY);
        picked.sort();
        picked.dedup();
        assert_eq!(picked.len(), 1000);
        assert!(available_buckets.is_none());
    }

    #[test]
    fn test_pick_twice() {
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let token_id = pick_token(&mut deps.storage, 975).unwrap();
        assert_eq!(token_id, 975);
        let res = pick_token(&mut deps.storage, 975);
        assert!(res.is_err());
    }
    #[test]
    fn test_purge_buckets() {
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let r = purge_buckets(&mut deps.storage, 10);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 10);
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY).unwrap();
        assert_eq!(available_buckets.len(), 30);
        assert_eq!(
            available_buckets,
            (0..30).map(|x| x as u8).collect::<Vec<u8>>()
        );
    }

    #[test]
    fn test_purge_all() {
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let r = purge_buckets(&mut deps.storage, 40);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 40);
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY);
        assert!(available_buckets.is_none());
    }
    #[test]
    fn test_shuffle() {
        let mut deps = mock_dependencies();
        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let r = shuffle(&mut deps.storage, [0; 32]);
        assert!(r.is_ok());
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY).unwrap();
        assert_eq!(available_buckets.len(), 40);
        assert_ne!(
            available_buckets,
            (0..40).map(|x| x as u8).collect::<Vec<u8>>()
        );
        assert_eq!(
            available_buckets,
            [
                21, 30, 37, 23, 11, 2, 29, 27, 8, 0, 7, 5, 15, 17, 6, 32, 25, 9, 36, 26, 13, 31,
                24, 10, 39, 35, 33, 12, 20, 16, 28, 18, 34, 19, 1, 4, 38, 22, 14, 3
            ]
        );
    }

    #[test]
    fn test_initialize_with_seed() {
        let mut deps = mock_dependencies();
        let r: Result<(), MinterUtilsError> =
            initialize_and_shuffle(&mut deps.storage, 10_000, [0; 32]);
        assert!(r.is_ok());
        let available_buckets = deps.storage.get(&AVAILABLE_BUCKETS_KEY).unwrap();
        assert_eq!(available_buckets.len(), 40);
        assert_ne!(
            available_buckets,
            (0..40).map(|x| x as u8).collect::<Vec<u8>>()
        );
        assert_eq!(
            available_buckets,
            [
                21, 30, 37, 23, 11, 2, 29, 27, 8, 0, 7, 5, 15, 17, 6, 32, 25, 9, 36, 26, 13, 31,
                24, 10, 39, 35, 33, 12, 20, 16, 28, 18, 34, 19, 1, 4, 38, 22, 14, 3
            ]
        );
    }

    #[test]
    fn test_shuffle_and_pick_token() {
        let mut deps = mock_dependencies();

        let r = initialize(&mut deps.storage, 10_000);
        assert!(r.is_ok());
        let r = shuffle(&mut deps.storage, [0; 32]);
        assert!(r.is_ok());
        let token_id = pick_token(&mut deps.storage, 975).unwrap();
        assert_eq!(token_id, 975);
    }
}
