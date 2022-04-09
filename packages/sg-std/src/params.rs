use cosmwasm_std::{Deps, StdResult};
use sg2::ParamResponseu32;

const PARAMSTORE_CONTRACT: &str =
    "stars1deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

/// Query a u32 value from the paramstore
/// Example:
/// `let contract_version = cw2::get_contract_version(deps)?.contract;`
/// `let param = query_param_u32(deps, contract_version, key)?;`
pub fn query_param_u32(
    deps: Deps,
    contract_name: String,
    key: String,
) -> StdResult<ParamResponseu32> {
    let msg = sg2::Sg2QueryMsg::GetParamu32 { contract_name, key };

    let res: ParamResponseu32 = deps.querier.query_wasm_smart(PARAMSTORE_CONTRACT, &msg)?;

    Ok(res)
}
