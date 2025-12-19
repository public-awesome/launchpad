use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg};
use crate::state::{increment_token_index, Config, COLLECTION_ADDRESS, CONFIG, STATUS};
use base_factory::msg::{BaseMinterCreateMsg, ParamsResponse};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo,
    Reply, Response, StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::set_contract_version;
use cw721::msg::{CollectionExtensionMsg, RoyaltyInfoResponse};
use cw721::Ownership;
use cw721_base::msg::{
    ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as Cw721InstantiateMsg, QueryMsg as Cw721QueryMsg,
};
use cw_utils::{must_pay, nonpayable, parse_instantiate_response_data};
use sg1::transfer_funds_to_launchpad_dao;
use sg2::query::Sg2QueryMsg;
use sg4::{QueryMsg, Status, StatusResponse, SudoMsg};
use sg_utils::FEE_DENOM;
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
    let offset = factory_params.params.max_trading_offset_secs;
    let start_trading_time = msg
        .collection_params
        .info
        .start_trading_time
        .or_else(|| Some(env.block.time.plus_seconds(offset)));

    // Convert CollectionExtension to CollectionExtensionMsg
    let collection_info_extension = CollectionExtensionMsg {
        description: Some(msg.collection_params.info.description.clone()),
        image: Some(msg.collection_params.info.image.clone()),
        external_link: msg.collection_params.info.external_link.clone(),
        explicit_content: msg.collection_params.info.explicit_content,
        start_trading_time,
        royalty_info: msg.collection_params.info.royalty_info.as_ref().map(|ri| {
            RoyaltyInfoResponse {
                payment_address: ri.payment_address.to_string(),
                share: ri.share,
            }
        }),
    };

    CONFIG.save(deps.storage, &config)?;

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_params.code_id,
        msg: to_json_binary(&Cw721InstantiateMsg {
            name: msg.collection_params.name.clone(),
            symbol: msg.collection_params.symbol.clone(),
            minter: Some(env.contract.address.to_string()),
            creator: Some(msg.collection_params.creator.clone()),
            collection_info_extension: Some(collection_info_extension),
            withdraw_address: None,
        })?,
        funds: info.funds,
        admin: Some(
            deps.api
                .addr_validate(&msg.collection_params.creator)?
                .to_string(),
        ),
        label: format!(
            "CW721-{}-{}",
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
    }
}

pub fn execute_mint_sender(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_uri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let collection_address = COLLECTION_ADDRESS.load(deps.storage)?;

    // This is a 1:1 minter, minted at min_mint_price
    // Should mint and then list on the marketplace for secondary sales
    let creator_ownership: Ownership<Addr> = deps.querier.query_wasm_smart(
        collection_address.clone(),
        &Cw721QueryMsg::GetCreatorOwnership {},
    )?;
    // allow only cw721 creator address to mint
    let creator = creator_ownership
        .owner
        .ok_or_else(|| ContractError::Unauthorized("No creator set".to_owned()))?;
    if creator != info.sender {
        return Err(ContractError::Unauthorized(
            "Sender is not cw721 creator".to_owned(),
        ));
    };

    // Token URI must be a valid URL (ipfs, https, etc.)
    Url::parse(&token_uri).map_err(|_| ContractError::InvalidTokenURI {})?;

    let mut res = Response::new();

    let factory: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &Sg2QueryMsg::Params {})?;
    let factory_params = factory.params;

    let funds_sent = must_pay(&info, FEE_DENOM)?;

    // Create network fee msgs
    let mint_fee_percent = Decimal::bps(factory_params.mint_fee_bps);
    let network_fee = config.mint_price.amount.mul_floor(mint_fee_percent);
    // For the base 1/1 minter, the entire mint price should be Fair Burned
    if network_fee != funds_sent {
        return Err(ContractError::InvalidMintPrice {});
    }

    transfer_funds_to_launchpad_dao(&info, network_fee.u128(), FEE_DENOM, &mut res)?;

    // Create mint msgs
    let mint_msg = Cw721ExecuteMsg::Mint {
        token_id: increment_token_index(deps.storage)?.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(token_uri.clone()),
        extension: None,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_address.to_string(),
        msg: to_json_binary(&mint_msg)?,
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
    let cw721_contract_addr = COLLECTION_ADDRESS.load(deps.storage)?;

    let creator_ownership: Ownership<Addr> = deps.querier.query_wasm_smart(
        cw721_contract_addr.clone(),
        &Cw721QueryMsg::GetCreatorOwnership {},
    )?;
    let creator = creator_ownership
        .owner
        .ok_or_else(|| ContractError::Unauthorized("No creator set".to_owned()))?;
    if info.sender != creator {
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

    // execute cw721 contract - update collection info with new start trading time
    let update_msg: CollectionExtensionMsg<RoyaltyInfoResponse> = CollectionExtensionMsg {
        description: None,
        image: None,
        external_link: None,
        explicit_content: None,
        royalty_info: None,
        start_trading_time: start_time,
    };
    let msg = WasmMsg::Execute {
        contract_addr: cw721_contract_addr.to_string(),
        msg: to_json_binary(&Cw721ExecuteMsg::UpdateCollectionInfo {
            collection_info: cw721::msg::CollectionInfoMsg {
                name: None,
                symbol: None,
                extension: Some(update_msg),
            },
        })?,
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
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Status {} => to_json_binary(&query_status(deps)?),
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

// Reply callback triggered from cw721 contract instantiation in instantiate()
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_SG721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let result = msg
        .result
        .into_result()
        .map_err(|_| ContractError::InstantiateSg721Error {})?;
    let data = result
        .msg_responses
        .first()
        .ok_or(ContractError::InstantiateSg721Error {})?
        .value
        .clone();

    let reply = parse_instantiate_response_data(&data);
    match reply {
        Ok(res) => {
            let collection_address = res.contract_address;
            COLLECTION_ADDRESS.save(deps.storage, &Addr::unchecked(collection_address.clone()))?;
            Ok(Response::default()
                .add_attribute("action", "instantiate_cw721_reply")
                .add_attribute("cw721_address", collection_address))
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}
