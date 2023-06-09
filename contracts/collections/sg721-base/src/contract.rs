use cosmwasm_std::{
    to_binary, Addr, Binary, ContractInfoResponse, Decimal, Deps, DepsMut, Empty, Env, Event,
    MessageInfo, StdError, StdResult, Storage, Timestamp, WasmQuery,
};
use cw721::{ContractInfoResponse as CW721ContractInfoResponse, Cw721Execute};
use cw721_base::state::TokenInfo;
use cw721_base::Extension;
use cw_utils::nonpayable;
use serde::{de::DeserializeOwned, Serialize};
use sg2::query::{ParamsResponse, Sg2QueryMsg};
use sg4::{MinterConfig, QueryMsg as MinterQueryMsg};
use sg721::{CollectionInfo, ExecuteMsg, InstantiateMsg, RoyaltyInfo, UpdateCollectionInfoMsg};
use sg_std::math::U64Ext;
use sg_std::Response;
use url::Url;

use crate::msg::{CollectionInfoResponse, NftParams, QueryMsg};
use crate::{ContractError, Sg721Contract};

use crate::entry::{CONTRACT_NAME, CONTRACT_VERSION, EARLIEST_VERSION, TO_VERSION};

const MAX_DESCRIPTION_LENGTH: u32 = 512;

impl<'a, T> Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // no funds should be sent to this contract
        nonpayable(&info)?;

        // check sender is a contract
        let req = WasmQuery::ContractInfo {
            contract_addr: info.sender.into(),
        }
        .into();
        let _res: ContractInfoResponse = deps
            .querier
            .query(&req)
            .map_err(|_| ContractError::Unauthorized {})?;

        // cw721 instantiation
        let info = CW721ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        self.parent.contract_info.save(deps.storage, &info)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.minter))?;

        // sg721 instantiation
        if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
            return Err(ContractError::DescriptionTooLong {});
        }

        let image = Url::parse(&msg.collection_info.image)?;

        if let Some(ref external_link) = msg.collection_info.external_link {
            Url::parse(external_link)?;
        }

        let royalty_info: Option<RoyaltyInfo<Addr>> = match msg.collection_info.royalty_info {
            Some(royalty_info) => Some(RoyaltyInfo {
                payment_address: deps.api.addr_validate(&royalty_info.payment_address)?,
                share: share_validate(royalty_info.share)?,
                updated_at: env.block.time,
            }),
            None => None,
        };

        deps.api.addr_validate(&msg.collection_info.creator)?;

        let collection_info = CollectionInfo {
            creator: msg.collection_info.creator,
            description: msg.collection_info.description,
            image: msg.collection_info.image,
            external_link: msg.collection_info.external_link,
            explicit_content: msg.collection_info.explicit_content,
            start_trading_time: msg.collection_info.start_trading_time,
            royalty_info,
            royalty_updated_at: Some(env.block.time),
        };

        self.collection_info.save(deps.storage, &collection_info)?;

        self.frozen_collection_info.save(deps.storage, &false)?;

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("collection_name", info.name)
            .add_attribute("image", image.to_string()))
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T, Empty>,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self
                .parent
                .transfer_nft(deps, env, info, recipient, token_id)
                .map_err(|e| e.into()),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self
                .parent
                .send_nft(deps, env, info, contract, token_id, msg)
                .map_err(|e| e.into()),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self
                .parent
                .approve(deps, env, info, spender, token_id, expires)
                .map_err(|e| e.into()),
            ExecuteMsg::Revoke { spender, token_id } => self
                .parent
                .revoke(deps, env, info, spender, token_id)
                .map_err(|e| e.into()),
            ExecuteMsg::ApproveAll { operator, expires } => self
                .parent
                .approve_all(deps, env, info, operator, expires)
                .map_err(|e| e.into()),
            ExecuteMsg::RevokeAll { operator } => self
                .parent
                .revoke_all(deps, env, info, operator)
                .map_err(|e| e.into()),
            ExecuteMsg::Burn { token_id } => self
                .parent
                .burn(deps, env, info, token_id)
                .map_err(|e| e.into()),
            ExecuteMsg::UpdateCollectionInfo { collection_info } => {
                self.update_collection_info(deps, env, info, collection_info)
            }
            ExecuteMsg::UpdateStartTradingTime(start_time) => {
                self.update_start_trading_time(deps, env, info, start_time)
            }
            ExecuteMsg::FreezeCollectionInfo {} => self.freeze_collection_info(deps, env, info),
            ExecuteMsg::Mint {
                token_id,
                token_uri,
                owner,
                extension,
            } => self.mint(
                deps,
                env,
                info,
                NftParams::NftData {
                    token_id,
                    owner,
                    token_uri,
                    extension,
                },
            ),
            ExecuteMsg::Extension { msg: _ } => todo!(),
            sg721::ExecuteMsg::UpdateOwnership(msg) => self
                .parent
                .execute(
                    deps,
                    env,
                    info,
                    cw721_base::ExecuteMsg::UpdateOwnership(msg),
                )
                .map_err(|e| ContractError::OwnershipUpdateError {
                    error: e.to_string(),
                }),
        }
    }

    pub fn update_collection_info(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collection_msg: UpdateCollectionInfoMsg<RoyaltyInfo<String>>,
    ) -> Result<Response, ContractError> {
        let mut collection = self.collection_info.load(deps.storage)?;

        if self.frozen_collection_info.load(deps.storage)? {
            return Err(ContractError::CollectionInfoFrozen {});
        }

        // only creator can update collection info
        if collection.creator != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        collection.description = collection_msg
            .description
            .unwrap_or_else(|| collection.description.to_string());
        if collection.description.len() > MAX_DESCRIPTION_LENGTH as usize {
            return Err(ContractError::DescriptionTooLong {});
        }

        collection.image = collection_msg
            .image
            .unwrap_or_else(|| collection.image.to_string());
        Url::parse(&collection.image)?;

        collection.external_link = collection_msg
            .external_link
            .unwrap_or_else(|| collection.external_link.as_ref().map(|s| s.to_string()));
        if collection.external_link.as_ref().is_some() {
            Url::parse(collection.external_link.as_ref().unwrap())?;
        }

        collection.explicit_content = collection_msg.explicit_content;

        // convert collection royalty info to response for comparison
        // convert from response to royalty info for storage
        let current_royalty_info = collection
            .royalty_info
            .map(|royalty_info| royalty_info.into());

        let new_royalty_info = collection_msg
            .royalty_info
            .unwrap_or_else(|| current_royalty_info.clone());

        // get the factory from the minter to get
        // max_royalty, max_royalty_increase_rate, royalty_min_time_duration_secs
        let minter_address = self.parent.minter(deps.as_ref())?.minter.unwrap();
        let minter_config: MinterConfig<T> = deps
            .querier
            .query_wasm_smart(minter_address, &MinterQueryMsg::Config {})?;
        let factory_address = minter_config.factory;
        let factory_params: ParamsResponse<T> = deps
            .querier
            .query_wasm_smart(factory_address, &Sg2QueryMsg::Params {})?;
        let max_royalty = factory_params.params.max_royalty_bps.bps_to_decimal();
        let max_royalty_increase_rate = factory_params
            .params
            .max_royalty_increase_rate_bps
            .bps_to_decimal();
        let royalty_min_time_duration = factory_params.params.royalty_min_time_duration_secs;

        let mut royalty_changed = false;
        // reminder: collection_msg.royalty_info is Option<Option<RoyaltyInfoResponse>>
        collection.royalty_info = if let Some(new_royalty_info_res) = new_royalty_info {
            // check if new_royalty_info_res > max_royalty
            if new_royalty_info_res.share > max_royalty {
                return Err(ContractError::RoyaltyShareTooHigh {});
            }
            // update royalty info to equal or less, else throw error
            if let Some(curr_royalty_info_res) = current_royalty_info {
                if new_royalty_info_res.share
                    > curr_royalty_info_res.share + max_royalty_increase_rate
                {
                    return Err(ContractError::RoyaltyShareIncreasedTooMuch {});
                }
                if new_royalty_info_res.share != curr_royalty_info_res.share {
                    royalty_changed = true;
                }
            } else {
                return Err(ContractError::RoyaltyShareIncreasedTooMuch {});
            }

            // if royalty share changed,
            // check if current time is after last royalty update + min duration
            // royalty_updated_at is always Some because it is set in instantiate
            if let Some(royalty_updated_at) = collection.royalty_updated_at {
                if royalty_changed
                    && royalty_updated_at.plus_seconds(royalty_min_time_duration) > env.block.time
                {
                    return Err(ContractError::RoyaltyUpdateTooSoon {});
                }
            }

            // set new updated_at if successful
            collection.royalty_updated_at = Some(env.block.time);
            Some(RoyaltyInfo {
                payment_address: deps
                    .api
                    .addr_validate(&new_royalty_info_res.payment_address)?,
                share: share_validate(new_royalty_info_res.share)?,
                updated_at: env.block.time,
            })
        } else {
            None
        };

        self.collection_info.save(deps.storage, &collection)?;

        let event = Event::new("update_collection_info").add_attribute("sender", info.sender);
        Ok(Response::new().add_event(event))
    }

    /// Called by the minter reply handler after custom validations on trading start time.
    /// Minter has start_time, default offset, makes sense to execute from minter.
    pub fn update_start_trading_time(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        start_time: Option<Timestamp>,
    ) -> Result<Response, ContractError> {
        assert_minter_owner(deps.storage, &info.sender)?;

        let mut collection_info = self.collection_info.load(deps.storage)?;
        collection_info.start_trading_time = start_time;
        self.collection_info.save(deps.storage, &collection_info)?;

        let event = Event::new("update_start_trading_time").add_attribute("sender", info.sender);
        Ok(Response::new().add_event(event))
    }

    pub fn freeze_collection_info(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let collection = self.query_collection_info(deps.as_ref())?;
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
        _env: Env,
        info: MessageInfo,
        nft_data: NftParams<T>,
    ) -> Result<Response, ContractError> {
        assert_minter_owner(deps.storage, &info.sender)?;
        let (token_id, owner, token_uri, extension) = match nft_data {
            NftParams::NftData {
                token_id,
                owner,
                token_uri,
                extension,
            } => (token_id, owner, token_uri, extension),
        };

        // create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&owner)?,
            approvals: vec![],
            token_uri: token_uri.clone(),
            extension,
        };
        self.parent
            .tokens
            .update(deps.storage, &token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;

        self.parent.increment_tokens(deps.storage)?;

        let mut res = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("owner", owner)
            .add_attribute("token_id", token_id);
        if let Some(token_uri) = token_uri {
            res = res.add_attribute("token_uri", token_uri);
        }
        Ok(res)
    }

    pub fn query(&self, deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::CollectionInfo {} => to_binary(&self.query_collection_info(deps)?),
            _ => self.parent.query(deps, env, msg.into()),
        }
    }

    pub fn query_collection_info(&self, deps: Deps) -> StdResult<CollectionInfoResponse> {
        let info = self.collection_info.load(deps.storage)?;

        let royalty_info_res: Option<RoyaltyInfo<String>> = match info.royalty_info {
            Some(royalty_info) => Some(RoyaltyInfo {
                payment_address: royalty_info.payment_address.to_string(),
                share: royalty_info.share,
                updated_at: royalty_info.updated_at,
            }),
            None => None,
        };

        Ok(CollectionInfoResponse {
            creator: info.creator,
            description: info.description,
            image: info.image,
            external_link: info.external_link,
            explicit_content: info.explicit_content,
            start_trading_time: info.start_trading_time,
            royalty_info: royalty_info_res,
        })
    }

    pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
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
        let cw17_res = cw721_base::upgrades::v0_17::migrate::<Extension, Empty, Empty, Empty>(deps)
            .map_err(|e| ContractError::MigrationError(e.to_string()))?;
        let mut sgz_res = Response::new();
        sgz_res.attributes = cw17_res.attributes;
        Ok(sgz_res)
    }
}

pub fn share_validate(share: Decimal) -> Result<Decimal, ContractError> {
    if share > Decimal::one() {
        return Err(ContractError::InvalidRoyalties {});
    }

    Ok(share)
}

pub fn get_owner_minter(storage: &mut dyn Storage) -> Result<Addr, ContractError> {
    let ownership = cw_ownable::get_ownership(storage)?;
    match ownership.owner {
        Some(owner_value) => Ok(owner_value),
        None => Err(ContractError::MinterNotFound {}),
    }
}

pub fn assert_minter_owner(storage: &mut dyn Storage, sender: &Addr) -> Result<(), ContractError> {
    let res = cw_ownable::assert_owner(storage, sender);
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(ContractError::UnauthorizedOwner {}),
    }
}
