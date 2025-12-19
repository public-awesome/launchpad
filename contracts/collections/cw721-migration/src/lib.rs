pub mod error;

pub use crate::error::ContractError;

// Re-export cw721 types for consumers
pub use cw721::{
    extension::Cw721BaseExtensions,
    state::{CollectionExtension, CollectionInfo, RoyaltyInfo},
    traits::{Cw721Execute, Cw721Query},
};

pub mod entry {
    use super::*;
    use cw721::traits::{Cw721Execute, Cw721Query};

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
    use cw2::set_contract_version;

    pub const CONTRACT_NAME: &str = "crates.io:cw721-migration";
    pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

    // Use Cw721BaseExtensions for collection metadata support
    pub type Cw721MigrationContract<'a> = Cw721BaseExtensions<'a>;

    // Type aliases for messages
    pub type InstantiateMsg = cw721_base::msg::InstantiateMsg;
    pub type ExecuteMsg = cw721_base::msg::ExecuteMsg;
    pub type QueryMsg = cw721_base::msg::QueryMsg;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let contract = Cw721MigrationContract::default();
        let res = contract.instantiate(deps, &env, &info, msg)?;

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
        let contract = Cw721MigrationContract::default();
        contract
            .execute(deps, &env, &info, msg)
            .map_err(|e| e.into())
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
        let contract = Cw721MigrationContract::default();
        contract.query(deps, &env, msg).map_err(|e| e.into())
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Ok(Response::default())
    }
}
