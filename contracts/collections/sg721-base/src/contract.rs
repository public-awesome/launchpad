use cw721_base::state::TokenInfo;
use url::Url;

use cosmwasm_std::{
    to_json_binary, Addr, Binary, ContractInfoResponse, Decimal, Deps, DepsMut, Empty, Env, Event,
    MessageInfo, Response, StdError, StdResult, Storage, Timestamp, WasmQuery,
};

use cw721::{ContractInfoResponse as CW721ContractInfoResponse, Cw721Execute};
use cw_utils::nonpayable;
use serde::{de::DeserializeOwned, Serialize};

use sg721::{
    CollectionInfo, ExecuteMsg, InstantiateMsg, RoyaltyInfo, RoyaltyInfoResponse,
    UpdateCollectionInfoMsg,
};

use crate::msg::{CollectionInfoResponse, NftParams, QueryMsg};
use crate::{ContractError, Sg721Contract};

use crate::entry::{CONTRACT_NAME, CONTRACT_VERSION};

const MAX_DESCRIPTION_LENGTH: u32 = 512;
const MAX_SHARE_DELTA_PCT: u64 = 2;
const MAX_ROYALTY_SHARE_PCT: u64 = 10;

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

        let royalty_info: Option<RoyaltyInfo> = match msg.collection_info.royalty_info {
            Some(royalty_info) => Some(RoyaltyInfo {
                payment_address: deps.api.addr_validate(&royalty_info.payment_address)?,
                share: share_validate(royalty_info.share)?,
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
        };

        self.collection_info.save(deps.storage, &collection_info)?;

        self.frozen_collection_info.save(deps.storage, &false)?;

        self.royalty_updated_at
            .save(deps.storage, &env.block.time)?;

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("collection_name", info.name)
            .add_attribute("collection_symbol", info.symbol)
            .add_attribute("collection_creator", collection_info.creator)
            .add_attribute("minter", msg.minter)
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
        collection_msg: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    ) -> Result<Response, ContractError> {
        let mut collection = self.collection_info.load(deps.storage)?;

        if self.frozen_collection_info.load(deps.storage)? {
            return Err(ContractError::CollectionInfoFrozen {});
        }

        // only creator can update collection info
        if collection.creator != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(new_creator) = collection_msg.creator {
            deps.api.addr_validate(&new_creator)?;
            collection.creator = new_creator;
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

        if let Some(Some(new_royalty_info_response)) = collection_msg.royalty_info {
            let last_royalty_update = self.royalty_updated_at.load(deps.storage)?;
            if last_royalty_update.plus_seconds(24 * 60 * 60) > env.block.time {
                return Err(ContractError::InvalidRoyalties(
                    "Royalties can only be updated once per day".to_string(),
                ));
            }

            let new_royalty_info = RoyaltyInfo {
                payment_address: deps
                    .api
                    .addr_validate(&new_royalty_info_response.payment_address)?,
                share: share_validate(new_royalty_info_response.share)?,
            };

            if let Some(old_royalty_info) = collection.royalty_info {
                if old_royalty_info.share < new_royalty_info.share {
                    let share_delta = new_royalty_info.share.abs_diff(old_royalty_info.share);

                    if share_delta > Decimal::percent(MAX_SHARE_DELTA_PCT) {
                        return Err(ContractError::InvalidRoyalties(format!(
                            "Share increase cannot be greater than {MAX_SHARE_DELTA_PCT}%"
                        )));
                    }
                    if new_royalty_info.share > Decimal::percent(MAX_ROYALTY_SHARE_PCT) {
                        return Err(ContractError::InvalidRoyalties(format!(
                            "Share cannot be greater than {MAX_ROYALTY_SHARE_PCT}%"
                        )));
                    }
                }
            }

            collection.royalty_info = Some(new_royalty_info);
            self.royalty_updated_at
                .save(deps.storage, &env.block.time)?;
        }

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
            QueryMsg::CollectionInfo {} => to_json_binary(&self.query_collection_info(deps)?),
            _ => self.parent.query(deps, env, msg.into()),
        }
    }

    pub fn query_collection_info(&self, deps: Deps) -> StdResult<CollectionInfoResponse> {
        let info = self.collection_info.load(deps.storage)?;

        let royalty_info_res: Option<RoyaltyInfoResponse> = match info.royalty_info {
            Some(royalty_info) => Some(RoyaltyInfoResponse {
                payment_address: royalty_info.payment_address.to_string(),
                share: royalty_info.share,
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

    pub fn migrate(mut deps: DepsMut, env: Env, _msg: Empty) -> Result<Response, ContractError> {
        let prev_contract_version = cw2::get_contract_version(deps.storage)?;

        let valid_contract_names = vec![CONTRACT_NAME.to_string()];
        if !valid_contract_names.contains(&prev_contract_version.contract) {
            return Err(StdError::generic_err("Invalid contract name for migration").into());
        }

        #[allow(clippy::cmp_owned)]
        if prev_contract_version.version >= CONTRACT_VERSION.to_string() {
            return Err(StdError::generic_err("Must upgrade contract version").into());
        }

        let mut response = Response::new();

        #[allow(clippy::cmp_owned)]
        if prev_contract_version.version < "3.0.0".to_string() {
            response = crate::upgrades::v3_0_0::upgrade(deps.branch(), &env, response)?;
        }

        #[allow(clippy::cmp_owned)]
        if prev_contract_version.version < "3.1.0".to_string() {
            response = crate::upgrades::v3_1_0::upgrade(deps.branch(), &env, response)?;
        }

        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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

pub fn share_validate(share: Decimal) -> Result<Decimal, ContractError> {
    if share > Decimal::one() {
        return Err(ContractError::InvalidRoyalties(
            "Share cannot be greater than 100%".to_string(),
        ));
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
