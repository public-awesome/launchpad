use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, ContractInfoResponse, CustomQuery, Deps, DepsMut, Querier,
    QuerierWrapper, StdError, StdResult, Timestamp, WasmMsg, WasmQuery,
};
use sg_std::CosmosMsg;
use sg_whitelist::msg::{
    ConfigResponse as WhitelistConfigResponse, HasMemberResponse, QueryMsg as WhitelistQueryMsg,
};
use sg_whitelist_merkle::msg::{
    ConfigResponse as MerkleWhitelistConfigResponse, HasMemberResponse as MerkleHasMemberResponse,
    QueryMsg as MerkleWhitelistQueryMsg,
};
use vending_factory::msg::{InitWhitelist, Whitelist};

use crate::{msg::ExecuteMsg, ContractError};

/// MinterContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterContract(pub Addr);

impl MinterContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn call_with_funds<T: Into<ExecuteMsg>>(
        &self,
        msg: T,
        funds: Coin,
    ) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![funds],
        }
        .into())
    }

    pub fn contract_info<Q, T, CQ>(&self, querier: &Q) -> StdResult<ContractInfoResponse>
    where
        Q: Querier,
        T: Into<String>,
        CQ: CustomQuery,
    {
        let query = WasmQuery::ContractInfo {
            contract_addr: self.addr().into(),
        }
        .into();
        let res: ContractInfoResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }
}

pub struct WhitelistConfig {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub is_active: bool,
    pub mint_price: Coin,
    pub per_address_limit: u32,
}

pub fn whitelist_config(deps: Deps, whitelist: Whitelist) -> Result<WhitelistConfig, StdError> {
    match whitelist {
        Whitelist::List { address } => {
            let res: WhitelistConfigResponse = deps
                .querier
                .query_wasm_smart(address, &WhitelistQueryMsg::Config {})?;
            Ok(WhitelistConfig {
                end_time: res.end_time,
                start_time: res.start_time,
                is_active: res.is_active,
                mint_price: res.mint_price,
                per_address_limit: res.per_address_limit,
            })
        }
        Whitelist::MerkleTree { address } => {
            let res: MerkleWhitelistConfigResponse = deps
                .querier
                .query_wasm_smart(address, &MerkleWhitelistQueryMsg::Config {})?;
            Ok(WhitelistConfig {
                end_time: res.end_time,
                start_time: res.start_time,
                is_active: res.is_active,
                mint_price: res.mint_price,
                per_address_limit: res.per_address_limit,
            })
        }
    }
}

pub fn parse_init_whitelist(deps: Deps, wl: InitWhitelist) -> Result<Whitelist, ContractError> {
    match wl {
        InitWhitelist::List { address } => {
            let addr = deps.api.addr_validate(&address)?;
            Ok(Whitelist::List { address: addr })
        }
        InitWhitelist::MerkleTree { address } => {
            let addr = deps.api.addr_validate(&address)?;
            Ok(Whitelist::MerkleTree { address: addr })
        }
    }
}

pub fn whitelist_exists(deps: DepsMut, wl: Whitelist) -> Result<(), ContractError> {
    match wl {
        Whitelist::List { address } => {
            let _: WhitelistConfigResponse = deps
                .querier
                .query_wasm_smart(address, &WhitelistQueryMsg::Config {})?;
            Ok(())
        }
        Whitelist::MerkleTree { address } => {
            let _: MerkleWhitelistConfigResponse = deps
                .querier
                .query_wasm_smart(address, &MerkleWhitelistQueryMsg::Config {})?;
            Ok(())
        }
    }
}
