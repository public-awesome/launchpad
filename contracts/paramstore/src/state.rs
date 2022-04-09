use cw_storage_plus::Map;

/// (contract_name, param_key) -> param_value
pub const U32: Map<(String, String), u32> = Map::new("u32");

// const U64: Map<&str, u64> = Map::new("u64");
// const UINT128: Map<&str, Uint128> = Map::new("uint128");
// const ADDR: Map<&str, Addr> = Map::new("addr");
