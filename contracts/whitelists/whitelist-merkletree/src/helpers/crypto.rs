use cosmwasm_std::{HexBinary, StdError, StdResult};

pub fn valid_hash_string(hash_string: &String) -> StdResult<()> {
    let hex_res = HexBinary::from_hex(hash_string.as_str());
    if hex_res.is_err() {
        return Err(cosmwasm_std::StdError::InvalidHex {
            msg: hash_string.to_string(),
        });
    }

    let hex_binary = hex_res.unwrap();

    let decoded = hex_binary.to_array::<32>();

    if decoded.is_err() {
        return Err(cosmwasm_std::StdError::InvalidDataSize {
            expected: 32,
            actual: hex_binary.len() as u64,
        });
    }
    Ok(())
}

pub fn verify_merkle_root(merkle_root: &String) -> StdResult<()> {
    valid_hash_string(merkle_root)
}

pub fn string_to_byte_slice(string: &String) -> StdResult<[u8; 32]> {
    let mut byte_slice = [0; 32];
    hex::decode_to_slice(string, &mut byte_slice).map_err(|_| StdError::GenericErr {
        msg: "Couldn't decode hash string".to_string(),
    })?;
    Ok(byte_slice)
}
