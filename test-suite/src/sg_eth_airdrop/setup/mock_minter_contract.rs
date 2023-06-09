use cosmwasm_std::{entry_point, Addr};
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, StdResult, Timestamp,
};
use cw_multi_test::{Contract, ContractWrapper};
use sg4::MinterConfig;
use sg_eth_airdrop::error::ContractError;
use sg_std::{Response, StargazeMsgWrapper};
use vending_factory::msg::VendingMinterCreateMsg;
use vending_minter::msg::{ConfigExtensionResponse, ExecuteMsg, QueryMsg};

use cosmwasm_schema::cw_serde;
#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub base_token_uri: String,
    pub num_tokens: u32,
    pub per_address_limit: u32,
    pub sg721_address: String,
    pub sg721_code_id: u64,
    pub start_time: Timestamp,
    pub mint_price: Coin,
    pub whitelist: Option<String>,
    pub factory: String,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: VendingMinterCreateMsg,
) -> Result<Response, ContractError> {
    let res = Response::new();
    Ok(res)
}

pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::CollectionWhitelistMinterNotSet {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config()),
        _ => to_binary("invalid"),
    }
}

fn query_config() -> MinterConfig<ConfigExtensionResponse> {
    MinterConfig {
        factory: Addr::unchecked("some_factory"),
        collection_code_id: 4,
        mint_price: Coin::new(1000, "ustars"),
        extension: ConfigExtensionResponse {
            admin: Addr::unchecked("some_admin"),
            collection_address: Some(Addr::unchecked("some_collection_address")),
            payment_address: Some(Addr::unchecked("some_payment_address")),
            base_token_uri: "some_uri".to_string(),
            num_tokens: 5,
            whitelist: Some(Addr::unchecked("contract2")),
            start_time: Timestamp::from_seconds(30),
            per_address_limit: 5,
            discount_price: Some(Coin::new(500, "ustars")),
        },
    }
}

pub fn mock_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}
