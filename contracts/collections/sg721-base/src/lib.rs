pub mod contract;
mod error;
pub mod msg;
mod state;
pub mod upgrades;

pub use crate::error::ContractError;
pub use crate::state::Sg721Contract;
use cosmwasm_std::Empty;
use cw721::{DefaultOptionalCollectionExtensionMsg, DefaultOptionalNftExtensionMsg};
use cw721_base::{DefaultOptionalCollectionExtension, DefaultOptionalNftExtension};

pub type ExecuteMsg =
    sg721::ExecuteMsg<DefaultOptionalNftExtensionMsg, DefaultOptionalCollectionExtensionMsg, Empty>;
pub type QueryMsg = cw721_base::msg::QueryMsg<
    DefaultOptionalNftExtension,
    DefaultOptionalCollectionExtension,
    Empty,
>;

pub mod entry {
    use super::*;
    use crate::{msg::QueryMsg, state::Sg721Contract};

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
    use cw2::set_contract_version;
    use cw721::msg::Cw721MigrateMsg;
    use sg721::InstantiateMsg;

    // version info for migration info
    pub const CONTRACT_NAME: &str = "crates.io:sg721-base";
    pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Sg721Contract::<
            DefaultOptionalNftExtension,
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtension,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
            Empty,
            Empty,
        >::default()
        .instantiate(deps, env, info, msg)?;

        Ok(res
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Sg721Contract::<
            DefaultOptionalNftExtension,
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtension,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
            Empty,
            Empty,
        >::default()
        .execute(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(
        deps: Deps,
        env: Env,
        msg: QueryMsg<DefaultOptionalNftExtension, DefaultOptionalCollectionExtension, Empty>,
    ) -> Result<Binary, ContractError> {
        Sg721Contract::<
            DefaultOptionalNftExtension,
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtension,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
            Empty,
            Empty,
        >::default()
        .query(deps, env, msg)
    }

    /// allows migration of all sg721-base versions
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(
        deps: DepsMut,
        env: Env,
        msg: Cw721MigrateMsg,
    ) -> Result<Response, ContractError> {
        Sg721Contract::<
            DefaultOptionalNftExtension,
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtension,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
            Empty,
            Empty,
        >::migrate(deps, env, msg)
    }
}
