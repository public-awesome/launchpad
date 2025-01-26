use cw721::msg::{CollectionInfoMsg, Cw721MigrateMsg};
use cw721::state::{MAX_COLLECTION_DESCRIPTION_LENGTH, MINTER};
use cw721::traits::{Contains, Cw721CustomMsg, Cw721State};
use cw721_base::msg::{CollectionExtensionMsg, RoyaltyInfoResponse};
use cw721_base::{
    traits::StateFactory, DefaultOptionalCollectionExtension, DefaultOptionalCollectionExtensionMsg,
};

use cosmwasm_std::{
    to_json_binary, Addr, Binary, ContractInfoResponse, CustomMsg, Deps, DepsMut, Env, Event,
    MessageInfo, Response, StdError, Storage, Timestamp, WasmQuery,
};

use cw721_base::traits::{Cw721Execute, Cw721Query};
use cw_utils::nonpayable;

#[allow(deprecated)]
use sg721::{ExecuteMsg, InstantiateMsg, UpdateCollectionInfoMsg};

#[allow(deprecated)]
use crate::msg::{CollectionInfoResponse, NftParams, QueryMsg};
use crate::{ContractError, Sg721Contract};

use crate::entry::{CONTRACT_NAME, CONTRACT_VERSION};

impl<
        'a,
        TNftExtension,
        TNftExtensionMsg,
        TExtensionMsg,
        TExtensionQueryMsg,
        TCustomResponseMsg,
    >
    Sg721Contract<
        'a,
        // NftInfo extension (onchain metadata).
        TNftExtension,
        // NftInfo extension msg for onchain metadata.
        TNftExtensionMsg,
        // CollectionInfo extension (onchain attributes).
        DefaultOptionalCollectionExtension,
        // CollectionInfo extension msg for onchain collection attributes.
        DefaultOptionalCollectionExtensionMsg,
        // Custom extension msg for custom contract logic. Default implementation is a no-op.
        TExtensionMsg,
        // Custom query msg for custom contract logic. Default implementation returns an empty binary.
        TExtensionQueryMsg,
        // Defines for `CosmosMsg::Custom<T>` in response. Barely used, so `Empty` can be used.
        TCustomResponseMsg,
    >
where
    TNftExtension: Cw721State + Contains,
    TNftExtensionMsg: Cw721CustomMsg + StateFactory<TNftExtension>,
    TExtensionQueryMsg: Cw721CustomMsg,
    TCustomResponseMsg: CustomMsg,
{
    #[allow(deprecated)]
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response<TCustomResponseMsg>, ContractError> {
        // no funds should be sent to this contract
        nonpayable(&info)?;

        // check sender is a contract
        let req = WasmQuery::ContractInfo {
            contract_addr: info.sender.to_string(),
        }
        .into();
        let _res: ContractInfoResponse = deps
            .querier
            .query(&req)
            .map_err(|_| ContractError::Unauthorized {})?;

        self.royalty_updated_at
            .save(deps.storage, &env.block.time)?;

        self.frozen_collection_info.save(deps.storage, &false)?;

        self.parent.instantiate_with_version(
            deps,
            &env,
            &info,
            msg.clone().into(),
            CONTRACT_NAME,
            CONTRACT_VERSION,
        )?;

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("collection_name", msg.name)
            .add_attribute("collection_symbol", msg.symbol)
            .add_attribute("collection_creator", msg.collection_info.creator)
            .add_attribute("minter", msg.minter)
            .add_attribute("image", msg.collection_info.image.to_string()))
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<TNftExtensionMsg, DefaultOptionalCollectionExtensionMsg, TExtensionMsg>,
    ) -> Result<Response<TCustomResponseMsg>, ContractError> {
        match msg {
            // ---- sg721 specific msgs ----
            #[allow(deprecated)]
            ExecuteMsg::UpdateCollectionInfo { collection_info } => {
                self.update_collection_info(deps, env, info, collection_info)
            }
            ExecuteMsg::UpdateStartTradingTime(start_time) => {
                self.update_start_trading_time(deps, env, info, start_time)
            }
            ExecuteMsg::FreezeCollectionInfo {} => self.freeze_collection_info(deps, env, info),
            // ---- cw721_base msgs ----
            msg => self
                .parent
                .execute(deps, &env, &info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[allow(deprecated)]
    pub fn update_collection_info(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collection_msg: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    ) -> Result<Response<TCustomResponseMsg>, ContractError> {
        let collection_info = self
            .parent
            .query_collection_info_and_extension(deps.as_ref())?;

        if self.frozen_collection_info.load(deps.storage)? {
            return Err(ContractError::CollectionInfoFrozen {});
        }

        // in this contract, extension is always present, so unwrap is safe
        let collection_extension = collection_info.extension.unwrap();

        if collection_extension.description.len() > MAX_COLLECTION_DESCRIPTION_LENGTH as usize {
            return Err(ContractError::DescriptionTooLong {});
        }

        if let Some(Some(_)) = collection_msg.royalty_info {
            let last_royalty_update = self.royalty_updated_at.load(deps.storage)?;
            if last_royalty_update.plus_seconds(24 * 60 * 60) > env.block.time {
                return Err(ContractError::InvalidRoyalties(
                    "Royalties can only be updated once per day".to_string(),
                ));
            }

            self.royalty_updated_at
                .save(deps.storage, &env.block.time)?;
        }

        let collection_extension: CollectionExtensionMsg<RoyaltyInfoResponse> =
            collection_msg.into();
        let msg = CollectionInfoMsg {
            name: None,
            symbol: None,
            extension: Some(collection_extension),
        };
        self.parent
            .update_collection_info(deps, Some(&info), Some(&env), msg)?;

        let event = Event::new("update_collection_info").add_attribute("sender", info.sender);
        Ok(Response::new().add_event(event))
    }

    /// Called by the minter reply handler after custom validations on trading start time.
    /// Minter has start_time, default offset, makes sense to execute from minter.
    pub fn update_start_trading_time(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        start_time: Option<Timestamp>,
    ) -> Result<Response<TCustomResponseMsg>, ContractError> {
        assert_minter_owner(deps.storage, &info.sender)?;

        let msg = CollectionInfoMsg {
            name: None,
            symbol: None,
            extension: Some(CollectionExtensionMsg {
                description: None,
                image: None,
                external_link: None,
                explicit_content: None,
                start_trading_time: start_time,
                royalty_info: None,
            }),
        };
        self.parent
            .update_collection_info(deps, Some(&info), Some(&env), msg)?;

        let event = Event::new("update_start_trading_time").add_attribute("sender", info.sender);
        Ok(Response::new().add_event(event))
    }

    pub fn freeze_collection_info(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response<TCustomResponseMsg>, ContractError> {
        let collection = self.query_collection_info(deps.as_ref())?;
        #[allow(deprecated)]
        if collection.creator != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let frozen = true;
        self.frozen_collection_info.save(deps.storage, &frozen)?;
        let event = Event::new("freeze_collection").add_attribute("sender", info.sender);
        Ok(Response::new().add_event(event))
    }

    pub fn mint(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        nft_data: NftParams<TNftExtensionMsg>,
    ) -> Result<Response<TCustomResponseMsg>, ContractError> {
        assert_minter_owner(deps.storage, &info.sender)?;
        let (token_id, owner, token_uri, extension) = match nft_data {
            NftParams::NftData {
                token_id,
                owner,
                token_uri,
                extension,
            } => (token_id, owner, token_uri, extension),
        };

        Ok(self
            .parent
            .mint(deps, &env, &info, token_id, owner, token_uri, extension)?)
    }

    pub fn get_creator(&self, storage: &dyn Storage) -> Result<Option<Addr>, ContractError> {
        // only creator can update collection info
        let creator = self.parent.query_creator_ownership(storage)?.owner;
        Ok(creator)
    }

    pub fn query(
        &self,
        deps: Deps,
        env: Env,
        msg: QueryMsg<TNftExtension, DefaultOptionalCollectionExtension, TExtensionQueryMsg>,
    ) -> Result<Binary, ContractError> {
        match msg {
            #[allow(deprecated)]
            QueryMsg::CollectionInfo {} => Ok(to_json_binary(&self.query_collection_info(deps)?)?),
            _ => Ok(self.parent.query(deps, &env, msg.into())?),
        }
    }

    #[allow(deprecated)]
    pub fn query_collection_info(
        &self,
        deps: Deps,
    ) -> Result<CollectionInfoResponse, ContractError> {
        let collection_info = self.parent.query_collection_info_and_extension(deps)?;

        let creator = self
            .get_creator(deps.storage)?
            .map_or("none".to_string(), |c| c.to_string());
        // in this contract, extension is always present, so unwrap is safe
        let collection_extension = collection_info.extension.unwrap();

        let collection_info = CollectionInfoResponse {
            creator,
            description: collection_extension.description,
            image: collection_extension.image,
            external_link: collection_extension.external_link,
            explicit_content: collection_extension.explicit_content,
            start_trading_time: collection_extension.start_trading_time,
            royalty_info: collection_extension.royalty_info.map(|r| r.into()),
        };

        Ok(collection_info)
    }

    pub fn migrate(
        mut deps: DepsMut,
        env: Env,
        msg: Cw721MigrateMsg,
    ) -> Result<Response, ContractError> {
        let prev_contract_version = cw2::get_contract_version(deps.storage)?;

        let valid_contract_names = vec![CONTRACT_NAME.to_string()];
        if !valid_contract_names.contains(&prev_contract_version.contract) {
            return Err(StdError::generic_err("Invalid contract name for migration").into());
        }

        let mut response = Response::new();

        // these upgrades can always be called. migration is only executed in case new stores are empty! It is safe calling these on any version.
        response = crate::upgrades::v3_0_0_ownable_and_collection_info::upgrade(
            deps.branch(),
            &env,
            response,
            msg,
        )?;
        response =
            crate::upgrades::v3_1_0_royalty_timestamp::upgrade(deps.branch(), &env, response)?;
        // after migration of collection metadata in cw721, we can migrate collection info to new collection extension
        response =
            crate::upgrades::v3_8_0_collection_metadata::upgrade(deps.branch(), &env, response)?;

        response = response.add_event(
            Event::new("migrate")
                .add_attribute("from_name", prev_contract_version.contract)
                .add_attribute("from_version", prev_contract_version.version)
                .add_attribute("to_name", CONTRACT_NAME)
                .add_attribute("to_version", CONTRACT_VERSION),
        );

        Ok(response)
    }
}

pub fn get_owner_minter(storage: &mut dyn Storage) -> Result<Addr, ContractError> {
    let ownership = MINTER.get_ownership(storage)?;
    match ownership.owner {
        Some(owner_value) => Ok(owner_value),
        None => Err(ContractError::MinterNotFound {}),
    }
}

pub fn assert_minter_owner(storage: &mut dyn Storage, sender: &Addr) -> Result<(), ContractError> {
    let res = MINTER.assert_owner(storage, sender);
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(ContractError::UnauthorizedOwner {}),
    }
}
