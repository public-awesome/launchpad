use cw721_base::state::TokenInfo;
use url::Url;

use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Empty, Env, Event, MessageInfo, StdResult,
};

use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw_utils::{nonpayable, Expiration};
use serde::{de::DeserializeOwned, Serialize};

use sg4::{MinterConfigResponse as Sg4MinterConfigResponse, QueryMsg as SG4QueryMsg};
use sg721::{
    CollectionInfo, ExecuteMsg, InstantiateMsg, MintMsg, RoyaltyInfo, RoyaltyInfoResponse,
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

        // cw721 instantiation
        let info = ContractInfoResponse {
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
            royalty_info,
        };

        self.collection_info.save(deps.storage, &collection_info)?;

        self.ready.save(deps.storage, &false)?;

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
        msg: ExecuteMsg<T>,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::_Ready {} => self.ready(deps, env, info),
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
            ExecuteMsg::Mint(msg) => self.mint(deps, env, info, msg),
        }
    }

    /// Called by the minter reply handler after instantiation. Now we can query
    /// the factory and minter to verify that the collection creation is authorized.
    pub fn ready(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let minter = self.parent.minter.load(deps.storage)?;
        if minter != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        // check if caller is a minter contract
        let _res: Sg4MinterConfigResponse<Empty> = deps
            .querier
            .query_wasm_smart(minter, &SG4QueryMsg::Config {})?;

        self.ready.save(deps.storage, &true)?;

        Ok(Response::new())
    }

    pub fn approve(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }

        self.parent
            ._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

        let event = Event::new("approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id);
        let res = Response::new().add_event(event);

        Ok(res)
    }

    pub fn approve_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // set the operator for us
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.parent
            .operators
            .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

        let event = Event::new("approve_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator);
        let res = Response::new().add_event(event);

        Ok(res)
    }

    pub fn burn(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
        let token = self.parent.tokens.load(deps.storage, &token_id)?;
        self.parent
            .check_can_send(deps.as_ref(), &env, &info, &token)?;

        self.parent.tokens.remove(deps.storage, &token_id)?;
        self.parent.decrement_tokens(deps.storage)?;

        let event = Event::new("burn")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id);
        let res = Response::new().add_event(event);

        Ok(res)
    }

    pub fn revoke(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
        self.parent
            ._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

        let event = Event::new("revoke")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id);
        let res = Response::new().add_event(event);

        Ok(res)
    }

    pub fn revoke_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.parent
            .operators
            .remove(deps.storage, (&info.sender, &operator_addr));

        let event = Event::new("revoke_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator);
        let res = Response::new().add_event(event);

        Ok(res)
    }

    pub fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        receiving_contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
        // Transfer token
        self.parent
            ._transfer_nft(deps, &env, &info, &receiving_contract, &token_id)?;

        let send = Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        };

        // Send message
        let event = Event::new("send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", receiving_contract.to_string())
            .add_attribute("token_id", token_id);
        let res = Response::new()
            .add_message(send.into_cosmos_msg(receiving_contract)?)
            .add_event(event);

        Ok(res)
    }

    pub fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
        self.parent
            ._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

        let event = Event::new("transfer_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id);
        let res = Response::new().add_event(event);

        Ok(res)
    }

    pub fn mint(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: MintMsg<T>,
    ) -> Result<Response, ContractError> {
        if !self.ready.load(deps.storage)? {
            return Err(ContractError::NotReady {});
        }
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
            royalty_info: royalty_info_res,
        })
    }
}

pub fn share_validate(share: Decimal) -> Result<Decimal, ContractError> {
    if share > Decimal::one() {
        return Err(ContractError::InvalidRoyalities {});
    }

    Ok(share)
}
