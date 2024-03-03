use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_json_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};

use crate::{
    msg::{ConfigResponse, QueryMsg},
    state::Config,
};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WhitelistImmutableFlexContract(pub Addr);

impl WhitelistImmutableFlexContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn includes(&self, querier: &QuerierWrapper, address: String) -> StdResult<bool> {
        let includes: bool = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::HasMember { address })?,
        }))?;
        Ok(includes)
    }

    pub fn address_count(&self, querier: &QuerierWrapper) -> StdResult<u64> {
        let address_count: u64 = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::AddressCount {})?,
        }))?;
        Ok(address_count)
    }

    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<Config> {
        let res: ConfigResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::Config {})?,
        }))?;

        Ok(res.config)
    }
}
