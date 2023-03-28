#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-nt";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod msg;
use cw721_base::Extension;
use sg721::InstantiateMsg;
use sg721_base::Sg721Contract;
pub type QueryMsg = sg721_base::msg::QueryMsg;
pub type Sg721NonTransferableContract<'a> = Sg721Contract<'a, Extension>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use crate::msg::ExecuteMsg;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
    use cw721::Cw721Execute;
    use sg721_base::ContractError;
    use sg_std::Response;

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Sg721NonTransferableContract::default().instantiate(deps, env, info, msg)?;

        Ok(res
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension>,
    ) -> Result<Response, sg721_base::ContractError> {
        match msg {
            ExecuteMsg::Burn { token_id } => Sg721NonTransferableContract::default()
                .parent
                .burn(deps, env, info, token_id)
                .map_err(|e| e.into()),
            ExecuteMsg::Mint(msg) => {
                Sg721NonTransferableContract::default().mint(deps, env, info, msg)
            }
            ExecuteMsg::UpdateCollectionInfo {
                new_collection_info,
            } => Sg721NonTransferableContract::default().update_collection_info(
                deps,
                env,
                info,
                new_collection_info,
            ),
            ExecuteMsg::FreezeCollectionInfo {} => {
                Sg721NonTransferableContract::default().freeze_collection_info(deps, env, info)
            }
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Sg721NonTransferableContract::default().query(deps, env, msg)
    }
}
