use anybuf::Bufany;
use cosmwasm_std::{
    coins, Addr, Api, BankMsg, Binary, BlockInfo, CustomMsg, CustomQuery, Empty, Querier, Storage,
};
use cw_multi_test::error::{bail, AnyResult};
use cw_multi_test::{AppResponse, CosmosRouter, Module, Stargate, StargateMsg, StargateQuery};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

pub struct StargazeKeeper<ExecT, QueryT, SudoT>(PhantomData<(ExecT, QueryT, SudoT)>);

impl<ExecT, QueryT, SudoT> StargazeKeeper<ExecT, QueryT, SudoT> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

pub type StargazeStargateKeeper = StargazeKeeper<StargateMsg, StargateQuery, Empty>;
impl Stargate for StargazeStargateKeeper {}

impl Module for StargazeStargateKeeper {
    // These associated types must match your type alias.
    type ExecT = StargateMsg;
    type QueryT = StargateQuery;
    type SudoT = Empty;

    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg.type_url.as_str() {
            "/publicawesome.stargaze.alloc.v1beta1.MsgFundFairburnPool" => {
                let decoded = Bufany::deserialize(&msg.value).unwrap();
                let amount_bytes = decoded.bytes(2).unwrap();

                let decoded_amount = Bufany::deserialize(&amount_bytes).unwrap();

                // field 1 is the denom
                // field 2 is the amount
                let denom = decoded_amount.string(1).unwrap();
                assert_eq!("ustars", denom);
                let amount = decoded_amount.string(2).unwrap();
                let msg = BankMsg::Send {
                    to_address: "fairburn_pool".to_owned(),
                    amount: coins(amount.parse::<u128>()?.into(), denom),
                }
                .into();
                let resp = router.execute(api, storage, block, sender, msg);
                match resp {
                    Ok(_) => return Ok(AppResponse::default()),
                    Err(e) => bail!("Error executing fairburn pool funding: {}", e),
                }
            }
            _ => bail!("stargate not implemented: {}", msg.type_url),
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _request: Self::QueryT,
    ) -> AnyResult<Binary> {
        Ok(Binary::default())
    }
}
