use cw721_base::state::TokenInfo;
use cw721_base::MintMsg;
use url::Url;

use cosmwasm_std::{
    to_binary, Binary, ContractInfoResponse, Decimal, Deps, DepsMut, Empty, Env, Event,
    MessageInfo, StdResult, Timestamp, WasmQuery,
};

use cw721::{ContractInfoResponse as CW721ContractInfoResponse, Cw721Execute};
use cw_utils::nonpayable;
use serde::{de::DeserializeOwned, Serialize};

use sg721::{
    CollectionInfo, ExecuteMsg, InstantiateMsg, RoyaltyInfo, RoyaltyInfoResponse,
    UpdateCollectionInfoMsg,
};
use sg_std::Response;

use crate::msg::{CollectionInfoResponse, QueryMsg};
use crate::{ContractError, Sg721Contract};

const MAX_DESCRIPTION_LENGTH: u32 = 512;

impl<'a, T> Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
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

        let minter = deps.api.addr_validate(&msg.minter)?;
        self.parent.minter.save(deps.storage, &minter)?;

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
            ExecuteMsg::Mint(msg) => self.mint(deps, env, info, msg),
            ExecuteMsg::Extension { msg: _ } => todo!(),
        }
    }

    pub fn update_collection_info(
        &self,
        deps: DepsMut,
        _env: Env,
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
        Url::parse(collection.external_link.as_ref().unwrap())?;

        collection.explicit_content = collection_msg.explicit_content;

        // convert collection royalty info to response for comparison
        // convert from response to royalty info for storage
        let royalty_info_res = collection
            .royalty_info
            .as_ref()
            .map(|royalty_info| royalty_info.to_response());

        let response = collection_msg
            .royalty_info
            .unwrap_or_else(|| royalty_info_res.clone());

        // reminder: collection_msg.royalty_info is Option<Option<RoyaltyInfoResponse>>
        collection.royalty_info = if let Some(royalty_info) = response {
            // update royalty info to equal or less, else throw error
            if let Some(royalty_info_res) = royalty_info_res {
                if royalty_info.share > royalty_info_res.share {
                    return Err(ContractError::RoyaltyShareIncreased {});
                }
            }

            Some(RoyaltyInfo {
                payment_address: deps.api.addr_validate(&royalty_info.payment_address)?,
                share: share_validate(royalty_info.share)?,
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
        let minter = self.parent.minter.load(deps.storage)?;
        if minter != info.sender {
            return Err(ContractError::Unauthorized {});
        }

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
        msg: MintMsg<T>,
    ) -> Result<Response, ContractError> {
        let minter = self.parent.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }

        // create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&msg.owner)?,
            approvals: vec![],
            token_uri: msg.token_uri,
            extension: msg.extension,
        };
        self.parent
            .tokens
            .update(deps.storage, &msg.token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;

        self.parent.increment_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("owner", msg.owner)
            .add_attribute("token_id", msg.token_id))
    }

    pub fn query(&self, deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::CollectionInfo {} => to_binary(&self.query_collection_info(deps)?),
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
}

pub fn share_validate(share: Decimal) -> Result<Decimal, ContractError> {
    if share > Decimal::one() {
        return Err(ContractError::InvalidRoyalties {});
    }

    Ok(share)
}
