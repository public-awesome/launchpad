use std::env;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, TokenUriMsg};
use crate::state::{increment_token_index, Config, COLLECTION_ADDRESS, CONFIG, STATUS};

use base_factory::msg::{BaseMinterCreateMsg, ParamsResponse};

use base_factory::state::Extension;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    StdResult, Timestamp, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;
use cw_utils::{must_pay, nonpayable, parse_reply_instantiate_data};

use sg1::checked_fair_burn;
use sg2::query::Sg2QueryMsg;
use sg2::{MinterParams, Token};
use sg4::{QueryMsg, Status, StatusResponse, SudoMsg};
use sg721::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::math::U64Ext;
use sg_std::{Response, SubMsg, NATIVE_DENOM};
use url::Url;

const CONTRACT_NAME: &str = "crates.io:sg-base-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_SG721_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseMinterCreateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let factory = info.sender.clone();

    // set default status so it can be queried without failing
    STATUS.save(deps.storage, &Status::default())?;

    // Make sure the sender is the factory contract
    // This will fail if the sender cannot parse a response from the factory contract
    let factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(factory.clone(), &Sg2QueryMsg::Params {})?;

    let config = Config {
        factory: factory.clone(),
        collection_code_id: msg.collection_params.code_id,
        // assume the mint price is the minimum mint price
        // 100% is fair burned
        mint_price: factory_params.params.min_mint_price,
        extension: Empty {},
    };

    // Use default start trading time if not provided
    let mut collection_info = msg.collection_params.info.clone();
    let offset = factory_params.params.max_trading_offset_secs;
    let start_trading_time = msg
        .collection_params
        .info
        .start_trading_time
        .or_else(|| Some(env.block.time.plus_seconds(offset)));
    collection_info.start_trading_time = start_trading_time;

    CONFIG.save(deps.storage, &config)?;

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_params.code_id,
        msg: to_binary(&Sg721InstantiateMsg {
            name: msg.collection_params.name.clone(),
            symbol: msg.collection_params.symbol,
            minter: env.contract.address.to_string(),
            collection_info,
        })?,
        funds: info.funds,
        admin: Some(
            deps.api
                .addr_validate(&msg.collection_params.info.creator)?
                .to_string(),
        ),
        label: format!(
            "SG721-{}-{}",
            msg.collection_params.code_id,
            msg.collection_params.name.trim()
        ),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INSTANTIATE_SG721_REPLY_ID);

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { token_uri } => execute_mint_sender(deps, env, info, token_uri),
        ExecuteMsg::UpdateStartTradingTime(time) => {
            execute_update_start_trading_time(deps, env, info, time)
        }
        ExecuteMsg::ReceiveNft(msg) => execute_burn_to_mint(deps, env, info, msg),
    }
}

// TODO: add a test to call `send` on the collection contract with a `Cw721ReceiveMsg`
// that includes a `TokenUriMsg` in the `msg` field

pub fn execute_burn_to_mint(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut res = Response::new();
    let burn_msg = cw721::Cw721ExecuteMsg::Burn {
        token_id: msg.token_id,
    };
    let cosmos_burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.sender.to_string(),
        msg: to_binary(&burn_msg)?,
        funds: vec![],
    });
    res = res.add_message(cosmos_burn_msg);

    let token_uri_msg: TokenUriMsg = from_binary(&msg.msg)?;
    let execute_mint_msg = ExecuteMsg::Mint {
        token_uri: token_uri_msg.token_uri,
    };
    let cosmos_mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&execute_mint_msg)?,
        funds: vec![],
    });
    let res = res.add_message(cosmos_mint_msg);
    Ok(res)
}

pub fn execute_mint_sender(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_uri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if matches!(config.mint_price, Token::NonFungible(_)) {
        return Err(ContractError::InvalidMintPrice {});
    }

    _execute_mint_sender(deps, env, info, token_uri)
}

fn _check_sender_is_collection_or_contract(
    this_contract: Addr,
    info: MessageInfo,
    collection_info: CollectionInfoResponse,
) -> Result<bool, ContractError> {
    let sender_is_collection_or_contract =
        vec![collection_info.creator, this_contract.to_string()].contains(&info.sender.to_string());
    if !(sender_is_collection_or_contract) {
        return Err(ContractError::Unauthorized(
            "Sender is not sg721 creator".to_owned(),
        ));
    }
    Ok(true)
}

fn _pay_mint_if_not_contract(
    this_contract: Addr,
    info: MessageInfo,
    mint_price: Token,
    factory_params: MinterParams<Option<Empty>>,
) -> Result<Uint128, ContractError> {
    let mut res = Response::new();
    let this_contract = this_contract.to_string();
    match info.clone().sender.to_string() == this_contract {
        true => Ok(0_u128.into()),
        false => {
            let funds_sent = must_pay(&info, NATIVE_DENOM)?;
            // Create network fee msgs
            let mint_fee_percent = factory_params.mint_fee_bps.bps_to_decimal();
            let network_fee = match mint_price {
                sg2::Token::Fungible(coin) => coin.amount * mint_fee_percent,
                sg2::Token::NonFungible(_) => Uint128::new(0),
            };
            // TODO: NFTs don't have a fee
            // For the base 1/1 minter, the entire mint price should be Fair Burned
            if network_fee != funds_sent {
                return Err(ContractError::InvalidMintPrice {});
            }
            checked_fair_burn(&info, network_fee.u128(), None, &mut res)?;
            Ok(network_fee)
        }
    }
}

fn _execute_mint_sender(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_uri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let collection_address = COLLECTION_ADDRESS.load(deps.storage)?;
    // This is a 1:1 minter, minted at min_mint_price
    // Should mint and then list on the marketplace for secondary sales
    let collection_info: CollectionInfoResponse = deps.querier.query_wasm_smart(
        collection_address.clone(),
        &Sg721QueryMsg::CollectionInfo {},
    )?;

    let _ = _check_sender_is_collection_or_contract(
        env.contract.address.clone(),
        info.clone(),
        collection_info.clone(),
    )?;

    let parsed_token_uri = Url::parse(&token_uri)?;
    if parsed_token_uri.scheme() != "ipfs" {
        return Err(ContractError::InvalidTokenURI {});
    }

    let mut res = Response::new();
    let factory: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &Sg2QueryMsg::Params {})?;
    let factory_params = factory.params;

    let network_fee = _pay_mint_if_not_contract(
        env.contract.address,
        info.clone(),
        config.mint_price,
        factory_params,
    )?;
    // Create mint msgs
    let mint_msg = Sg721ExecuteMsg::<Extension, Empty>::Mint {
        token_id: increment_token_index(deps.storage)?.to_string(),
        owner: collection_info.creator,
        token_uri: Some(token_uri.clone()),
        extension: None,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    res = res.add_message(msg);

    Ok(res
        .add_attribute("action", "mint")
        .add_attribute("sender", info.sender)
        .add_attribute("token_uri", token_uri)
        .add_attribute("network_fee", network_fee.to_string()))
}

pub fn execute_update_start_trading_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<Timestamp>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let sg721_contract_addr = COLLECTION_ADDRESS.load(deps.storage)?;

    let collection_info: CollectionInfoResponse = deps.querier.query_wasm_smart(
        sg721_contract_addr.clone(),
        &Sg721QueryMsg::CollectionInfo {},
    )?;
    if info.sender != collection_info.creator {
        return Err(ContractError::Unauthorized(
            "Sender is not creator".to_owned(),
        ));
    }

    // add custom rules here
    if let Some(start_time) = start_time {
        if env.block.time > start_time {
            return Err(ContractError::InvalidStartTradingTime(
                env.block.time,
                start_time,
            ));
        }
    }

    // execute sg721 contract
    let msg = WasmMsg::Execute {
        contract_addr: sg721_contract_addr.to_string(),
        msg: to_binary(&Sg721ExecuteMsg::<Extension, Empty>::UpdateStartTradingTime(start_time))?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("sender", info.sender)
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateStatus {
            is_verified,
            is_blocked,
            is_explicit,
        } => update_status(deps, is_verified, is_blocked, is_explicit)
            .map_err(|_| ContractError::UpdateStatus {}),
    }
}

/// Only governance can update contract params
pub fn update_status(
    deps: DepsMut,
    is_verified: bool,
    is_blocked: bool,
    is_explicit: bool,
) -> StdResult<Response> {
    let mut status = STATUS.load(deps.storage)?;
    status.is_verified = is_verified;
    status.is_blocked = is_blocked;
    status.is_explicit = is_explicit;
    STATUS.save(deps.storage, &status)?;

    Ok(Response::new().add_attribute("action", "sudo_update_status"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Status {} => to_binary(&query_status(deps)?),
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

pub fn query_status(deps: Deps) -> StdResult<StatusResponse> {
    let status = STATUS.load(deps.storage)?;

    Ok(StatusResponse { status })
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
            COLLECTION_ADDRESS.save(deps.storage, &Addr::unchecked(collection_address.clone()))?;
            Ok(Response::default()
                .add_attribute("action", "instantiate_sg721_reply")
                .add_attribute("sg721_address", collection_address))
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}
