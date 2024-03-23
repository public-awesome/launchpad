#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub mod msg;
use cosmwasm_std::Empty;
use cw721::{
    DefaultOptionCollectionMetadataExtension, DefaultOptionCollectionMetadataExtensionMsg,
    DefaultOptionNftMetadataExtension, DefaultOptionNftMetadataExtensionMsg,
};
use sg721::InstantiateMsg;
use sg721_base::Sg721Contract;
pub type QueryMsg = sg721_base::msg::QueryMsg<
    DefaultOptionNftMetadataExtension,
    DefaultOptionCollectionMetadataExtension,
>;
pub type Sg721NonTransferableContract<'a> = Sg721Contract<
    'a,
    DefaultOptionNftMetadataExtension,
    DefaultOptionNftMetadataExtensionMsg,
    DefaultOptionCollectionMetadataExtension,
    DefaultOptionCollectionMetadataExtensionMsg,
    Empty,
>;
use sg721_base::msg::NftParams;

// version info for migration info
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-nt";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const EARLIEST_VERSION: &str = "0.16.0";
pub const TO_VERSION: &str = "3.0.0";

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use crate::msg::ExecuteMsg;
    use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError};
    use cw721::msg::Cw721MigrateMsg;
    use cw721_base::traits::Cw721Execute;
    use sg721_base::ContractError;

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
        msg: ExecuteMsg<DefaultOptionNftMetadataExtension>,
    ) -> Result<Response, sg721_base::ContractError> {
        match msg {
            ExecuteMsg::Burn { token_id } => Sg721NonTransferableContract::default()
                .parent
                .burn_nft(deps, &env, &info, token_id)
                .map_err(|e| e.into()),
            ExecuteMsg::Mint {
                token_id,
                token_uri,
                owner,
                extension,
            } => {
                let nft_data = NftParams::NftData {
                    token_id,
                    owner,
                    token_uri,
                    extension,
                };
                Sg721NonTransferableContract::default().mint(deps, env, info, nft_data)
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
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
        Sg721NonTransferableContract::default().query(deps, env, msg)
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, env: Env, _msg: Empty) -> Result<Response, ContractError> {
        // make sure the correct contract is being upgraded, and it's being
        // upgraded from the correct version.
        if CONTRACT_VERSION < EARLIEST_VERSION {
            return Err(
                StdError::generic_err("Cannot upgrade to a previous contract version").into(),
            );
        }
        if CONTRACT_VERSION > TO_VERSION {
            return Err(
                StdError::generic_err("Cannot upgrade to a previous contract version").into(),
            );
        }
        // if same version return
        if CONTRACT_VERSION == TO_VERSION {
            return Ok(Response::new());
        }

        // update contract version
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, TO_VERSION)?;

        // perform the upgrade
        // cw721 migration allows all versions: 0.18. 0.17, 0.16 and older
        let contract = Sg721Contract::<
            DefaultOptionNftMetadataExtension,
            DefaultOptionNftMetadataExtensionMsg,
            DefaultOptionCollectionMetadataExtension,
            DefaultOptionCollectionMetadataExtensionMsg,
            Empty,
        >::default();
        let migrate_msg = Cw721MigrateMsg::WithUpdate {
            minter: None,
            creator: None,
        };
        let cw721_res = contract
            .parent
            .migrate(
                deps,
                env.clone(),
                migrate_msg,
                CONTRACT_NAME,
                CONTRACT_VERSION,
            )
            .map_err(|e| ContractError::MigrationError(e.to_string()))?;
        let mut sgz_res = Response::new();
        sgz_res.attributes = cw721_res.attributes;
        Ok(sgz_res)
    }
}
