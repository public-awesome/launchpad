use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, QueryMsg};
use crate::state::{increment_token_index, Config, COLLECTION_ADDRESS, CONFIG};

use base_factory::msg::{BaseMinterCreateMsg, Extension, ParamsResponse};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    StdResult, WasmMsg,
};

use cw2::set_contract_version;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_utils::{must_pay, parse_reply_instantiate_data};

use sg1::checked_fair_burn;
use sg2::query::Sg2QueryMsg;
use sg721::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg_std::math::U64Ext;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};
use url::Url;

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-base-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_SG721_REPLY_ID: u64 = 1;

// TODO: no need to have controllers when we have base contracts

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseMinterCreateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let factory = info.sender.clone();

    // TODO: set Status here (to false for all)

    // Make sure the sender is the factory contract
    // This will fail if the sender cannot parse a response from the factory contract
    let res: ParamsResponse = deps
        .querier
        .query_wasm_smart(factory.clone(), &Sg2QueryMsg::Params {})?;

    let config = Config {
        factory: factory.clone(),
        collection_code_id: msg.collection_params.code_id,
        // assume the mint price is the minimum mint price
        // 100% is fair burned
        mint_price: res.params.min_mint_price,
        extension: Empty {},
    };

    CONFIG.save(deps.storage, &config)?;

    // Submessage to instantiate sg721 contract
    let submsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.collection_params.code_id,
            msg: to_binary(&Sg721InstantiateMsg {
                name: msg.collection_params.name.clone(),
                symbol: msg.collection_params.symbol,
                minter: env.contract.address.to_string(),
                collection_info: msg.collection_params.info,
            })?,
            funds: info.funds,
            admin: None,
            label: format!("SG721-{}", msg.collection_params.name),
        }
        .into(),
        id: INSTANTIATE_SG721_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", factory)
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { token_uri } => execute_mint_sender(deps, info, token_uri),
    }
}

pub fn execute_mint_sender(
    deps: DepsMut,
    info: MessageInfo,
    token_uri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let collection = COLLECTION_ADDRESS.load(deps.storage)?;

    let parsed_token_uri = Url::parse(&token_uri)?;
    if parsed_token_uri.scheme() != "ipfs" {
        return Err(ContractError::InvalidTokenURI {});
    }

    let mut res = Response::new();

    let factory: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &Sg2QueryMsg::Params {})?;
    let factory_params = factory.params;

    let funds_sent = must_pay(&info, NATIVE_DENOM)?;

    // Create network fee msgs
    let mint_fee_percent = factory_params.mint_fee_bps.bps_to_decimal();
    let network_fee = config.mint_price.amount * mint_fee_percent;
    // For the base 1/1 minter, the entire mint price should be Fair Burned
    if network_fee != funds_sent {
        return Err(ContractError::InvalidMintPrice {});
    }
    checked_fair_burn(&info, network_fee.u128(), None, &mut res)?;

    // Create mint msgs
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: increment_token_index(deps.storage)?.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(token_uri),
        extension: None,
    });
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    res = res.add_message(msg);

    Ok(res
        .add_attribute("action", "mint")
        .add_attribute("sender", info.sender)
        .add_attribute("network_fee", network_fee.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let collection_address = COLLECTION_ADDRESS.load(deps.storage)?;

    Ok(ConfigResponse {
        collection_address: collection_address.to_string(),
        config,
    })
}

// Reply callback triggered from sg721 contract instantiation in instantiate()
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_SG721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_address = res.contract_address;

            // mark the collection contract as ready to mint
            let msg = WasmMsg::Execute {
                contract_addr: collection_address.clone(),
                msg: to_binary(&Sg721ExecuteMsg::<Empty>::_Ready {})?,
                funds: vec![],
            };

            COLLECTION_ADDRESS.save(deps.storage, &Addr::unchecked(collection_address))?;

            Ok(Response::default()
                .add_attribute("action", "instantiate_sg721_reply")
                .add_message(msg))
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}
