#[cfg(not(feature = "library"))]
pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub type InstantiateMsg = sg721::InstantiateMsg;

pub mod entry {
    use super::*;
    use crate::error::ContractError;
    use crate::msg::QueryMsg;
    use crate::{
        contract::{
            _instantiate, _migrate, execute_enable_updatable, execute_freeze_token_metadata,
            execute_update_token_metadata, query_enable_updatable, query_enable_updatable_fee,
            query_frozen_token_metadata, Sg721UpdatableContract,
        },
        msg::ExecuteMsg,
    };
    use cosmwasm_std::{entry_point, to_json_binary};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
    use cw721::msg::Cw721MigrateMsg;
    use cw721::{
        DefaultOptionCollectionMetadataExtensionMsg, DefaultOptionNftMetadataExtensionMsg,
    };

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
        msg: ExecuteMsg<
            DefaultOptionNftMetadataExtensionMsg,
            DefaultOptionCollectionMetadataExtensionMsg,
        >,
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
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
        match msg {
            QueryMsg::EnableUpdatable {} => Ok(to_json_binary(&query_enable_updatable(deps)?)?),
            QueryMsg::EnableUpdatableFee {} => Ok(to_json_binary(&query_enable_updatable_fee()?)?),
            QueryMsg::FreezeTokenMetadata {} => {
                Ok(to_json_binary(&query_frozen_token_metadata(deps)?)?)
            }
            _ => Ok(Sg721UpdatableContract::default().query(deps, env, msg.into())?),
        }
    }

    #[entry_point]
    pub fn migrate(
        deps: DepsMut,
        env: Env,
        msg: Cw721MigrateMsg,
    ) -> Result<Response, ContractError> {
        _migrate(deps, env, msg)
    }
}
