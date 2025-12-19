use anybuf::Bufany;
use cosmwasm_std::{
    coins, Addr, AnyMsg, Api, BankMsg, Binary, BlockInfo, CustomMsg, CustomQuery, Querier, Storage,
};
use cw_multi_test::error::{bail, AnyResult};
use cw_multi_test::{AppResponse, CosmosRouter, Stargate};
use serde::de::DeserializeOwned;
use sg_utils::NATIVE_DENOM;

pub struct StargazeStargateKeeper;

impl StargazeStargateKeeper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StargazeStargateKeeper {
    fn default() -> Self {
        Self::new()
    }
}

impl Stargate for StargazeStargateKeeper {
    fn execute_any<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: AnyMsg,
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
                assert_eq!(NATIVE_DENOM, denom);
                let amount = decoded_amount.string(2).unwrap();
                let bank_msg = BankMsg::Send {
                    to_address: "fairburn_pool".to_owned(),
                    amount: coins(amount.parse::<u128>()?, denom),
                }
                .into();
                let resp = router.execute(api, storage, block, sender, bank_msg);
                match resp {
                    Ok(_) => Ok(AppResponse::default()),
                    Err(e) => bail!("Error executing fairburn pool funding: {}", e),
                }
            }
            _ => bail!("stargate not implemented: {}", msg.type_url),
        }
    }

    fn query_grpc(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _request: cosmwasm_std::GrpcQuery,
    ) -> AnyResult<Binary> {
        Ok(Binary::default())
    }
}
