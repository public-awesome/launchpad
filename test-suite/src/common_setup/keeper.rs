use anybuf::Bufany;
use anyhow::bail;
use cosmwasm_std::{coins, Addr, Api, BankMsg, Binary, BlockInfo, CustomQuery, Querier, Storage};
use cw_multi_test::error::AnyResult;
use cw_multi_test::{AppResponse, CosmosRouter, Stargate};
use serde::de::DeserializeOwned;

use sg_std::NATIVE_DENOM;

pub struct StargazeKeeper;

impl Stargate for StargazeKeeper {
    /// Custom processing of stargate messages.
    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        type_url: String,
        value: Binary,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Clone + PartialEq + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match type_url.as_str() {
            "/publicawesome.stargaze.alloc.v1beta1.MsgFundFairburnPool" => {
                let decoded = Bufany::deserialize(&value).unwrap();
                let amount_bytes = decoded.bytes(2).unwrap();

                let decoded_amount = Bufany::deserialize(&amount_bytes).unwrap();

                // field 1 is the denom
                // field 2 is the amount
                let denom = decoded_amount.string(1).unwrap();
                assert_eq!(NATIVE_DENOM, denom);
                let amount = decoded_amount.string(2).unwrap();
                let msg = BankMsg::Send {
                    to_address: "fairburn_pool".to_owned(),
                    amount: coins(amount.parse::<u128>()?, denom),
                }
                .into();
                let resp = router.execute(api, storage, block, sender, msg);
                match resp {
                    Ok(_) => Ok(AppResponse::default()),
                    Err(e) => bail!("Error executing fairburn pool funding: {}", e),
                }
            }
            _ => {
                bail!(
                    "Unexpected stargate message: (type_url = {}, value = {:?}) from {:?}",
                    type_url,
                    value,
                    sender
                )
            }
        }
    }

    /// Custom stargate queries.
    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _path: String,
        _data: Binary,
    ) -> AnyResult<Binary> {
        bail!("not implemented");
    }
}
