use cw721::NumTokensResponse;
use launchpad::ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, ContractInfoResponse, CustomQuery, Querier, QuerierWrapper, StdResult,
    WasmMsg, WasmQuery,
};
use sg_std::CosmosMsg;

use crate::msg::QueryMsg;

/// Sg721VendingContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Sg721VendingContract(pub Addr);

impl Sg721VendingContract {
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

    pub fn num_tokens<Q, CQ>(&self, querier: &Q) -> StdResult<NumTokensResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::NumTokens {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();
        let res: NumTokensResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
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
