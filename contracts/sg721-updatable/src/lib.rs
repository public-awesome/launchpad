#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub type InstantiateMsg = sg721::InstantiateMsg;
pub type QueryMsg = sg721_base::msg::QueryMsg;

pub mod entry {
    use super::*;
    use crate::error::ContractError;
    use crate::{
        contract::{
            _instantiate, execute_freeze_token_metadata, execute_update_token_metadata,
            Sg721UpdatableContract,
        },
        msg::ExecuteMsg,
    };
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
    use cw721_base::Extension;
    use sg_std::Response;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        _instantiate(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension>,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::FreezeTokenMetadata {} => execute_freeze_token_metadata(deps, env, info),
            ExecuteMsg::UpdateTokenMetadata {
                token_id,
                token_uri,
            } => execute_update_token_metadata(deps, env, info, token_id, token_uri),
            _ => Sg721UpdatableContract::default()
                .execute(deps, env, info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Sg721UpdatableContract::default().query(deps, env, msg)
    }
}
