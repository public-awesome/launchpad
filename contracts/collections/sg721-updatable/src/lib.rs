#[cfg(not(feature = "library"))]
pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub type InstantiateMsg = sg721::InstantiateMsg;

pub mod entry {
    use super::*;
    use crate::error::ContractError;
    use crate::msg::{QueryMsg, SudoMsg};
    use crate::{
        contract::{
            _instantiate, _migrate, execute_enable_updatable, execute_freeze_token_metadata,
            execute_update_token_metadata, query_enable_updatable, query_frozen_token_metadata,
            sudo_update_params, Sg721UpdatableContract,
        },
        msg::ExecuteMsg,
    };
    use cosmwasm_std::{entry_point, to_binary, Empty};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
    use cw721_base::Extension;

    use sg_std::Response;

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        _instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension, Empty>,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::FreezeTokenMetadata {} => execute_freeze_token_metadata(deps, env, info),
            ExecuteMsg::EnableUpdatable {} => execute_enable_updatable(deps, env, info),
            ExecuteMsg::UpdateTokenMetadata {
                token_id,
                token_uri,
            } => execute_update_token_metadata(deps, env, info, token_id, token_uri),
            _ => Sg721UpdatableContract::default()
                .execute(deps, env, info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::EnableUpdatable {} => to_binary(&query_enable_updatable(deps)?),
            QueryMsg::FreezeTokenMetadata {} => to_binary(&query_frozen_token_metadata(deps)?),
            _ => Sg721UpdatableContract::default().query(deps, env, msg.into()),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
        match msg {
            SudoMsg::UpdateParams(params_msg) => sudo_update_params(deps, env, *params_msg),
        }
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, env: Env, msg: Empty) -> Result<Response, ContractError> {
        _migrate(deps, env, msg)
    }
}
