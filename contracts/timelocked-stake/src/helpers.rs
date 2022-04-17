// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};

// use cosmwasm_std::{
//     to_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
// };

// use crate::msg::{ExecuteMsg, QueryMsg, StakeResponse};

// /// SgStakingContract is a wrapper around Addr that provides a lot of helpers
// /// for working with this.
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct SgStakingContract(pub Addr);

// impl SgStakingContract {
//     pub fn addr(&self) -> Addr {
//         self.0.clone()
//     }

//     pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
//         let msg = to_binary(&msg.into())?;
//         Ok(WasmMsg::Execute {
//             contract_addr: self.addr().into(),
//             msg,
//             funds: vec![],
//         }
//         .into())
//     }

//     /// Get stake
//     pub fn stake<Q, T, CQ>(&self, querier: &Q, address: String) -> StdResult<StakeResponse>
//     where
//         Q: Querier,
//         T: Into<String>,
//         CQ: CustomQuery,
//     {
//         let msg = QueryMsg::Stake {};
//         let query = WasmQuery::Smart {
//             contract_addr: self.addr().into(),
//             msg: to_binary(&msg)?,
//         }
//         .into();
//         let res: StakeResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
//         Ok(res)
//     }
// }
