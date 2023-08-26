use cosmwasm_std::{HexBinary, StdResult};

pub fn valid_hash_string(hash_string: &String) -> StdResult<()> {
    let hex_res = HexBinary::from_hex(hash_string.as_str());
    if hex_res.is_err() {
        return Err(cosmwasm_std::StdError::InvalidHex { msg: hash_string.to_string() });
    }
    let decoded = hex_res.unwrap().to_array::<32>();

    if decoded.is_err() {
        return Err(cosmwasm_std::StdError::InvalidDataSize { expected: 32, actual: hash_string.as_bytes().len() as u64 })
    }
    Ok(())
}

pub fn verify_merkle_root(merkle_root: &String) -> StdResult<()> {
    valid_hash_string(merkle_root)
}


// pub fn check_merkle_tree_inclusion(string: Addr) -> StdResult<()> {
//     admins.iter().map(|addr| api.addr_validate(addr)).collect()
// }
