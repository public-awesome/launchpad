use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BankMsg, Coin, coins, ContractInfoResponse, CustomQuery, Decimal, Deps, Event, Querier, QuerierWrapper, StdResult, to_binary, Uint128, WasmMsg, WasmQuery};
use sg_std::{CosmosMsg, NATIVE_DENOM, Response, SubMsg};

use crate::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, QueryMsg};

/// MinterContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
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

    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let res: ConfigResponse = querier.query_wasm_smart(self.addr(), &QueryMsg::Config {})?;
        Ok(res)
    }
}

pub fn dev_fee_msgs_and_amount(
    deps: Deps,
    sale_price: Uint128,
    dev_fee_decimal: Decimal,
    developer: String,
    res: &mut Response
) -> Result<Uint128, ContractError> {
    let mut event = Event::new("dev-fees");

    let dev_fee = (sale_price * dev_fee_decimal).u128();
    res.messages.push(SubMsg::new(BankMsg::Send {
        to_address: deps.api.addr_validate(&developer)?.to_string(),
        amount: coins(dev_fee, NATIVE_DENOM),
    }));
    event = event.add_attribute("dev", developer);
    event = event.add_attribute("dev_amount", Uint128::from(dev_fee).to_string());
    res.events.push(event);

    Ok(Uint128::new(dev_fee))
}
