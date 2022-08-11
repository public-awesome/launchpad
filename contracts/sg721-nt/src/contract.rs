use sg721_base::contract::create;

use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;

use sg721::InstantiateMsg;
use sg_std::Response;

use crate::Cw721Base;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-nt";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn _instantiate(
    contract: Cw721Base,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, sg721_base::ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    create(contract, deps, info, msg)
}
