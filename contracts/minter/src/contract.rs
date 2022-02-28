#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, ReplyOn, StdError, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721::TokensResponse as Cw721TokensResponse;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_utils::{may_pay, parse_reply_instantiate_data, Expiration};
use sg721::msg::{InstantiateMsg as Sg721InstantiateMsg, QueryMsg as Sg721QueryMsg};
use url::Url;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MintableNumTokensResponse, QueryMsg,
    StartTimeResponse,
};
use crate::state::{Config, CONFIG, MINTABLE_TOKEN_IDS, SG721_ADDRESS};
use sg_std::{burn_and_distribute_fee, StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use whitelist::msg::{
    HasEndedResponse, HasMemberResponse, HasStartedResponse, QueryMsg as WhitelistQueryMsg,
    UnitPriceResponse,
};

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_SG721_REPLY_ID: u64 = 1;

// governance parameters
const MAX_TOKEN_LIMIT: u32 = 10000;
const MAX_PER_ADDRESS_LIMIT: u64 = 30;
const MAX_BATCH_MINT_LIMIT: u64 = 30;
const STARTING_BATCH_MINT_LIMIT: u64 = 5;
const STARTING_PER_ADDRESS_LIMIT: u64 = 5;
const MIN_MINT_PRICE: u128 = 50_000_000;
const MINT_FEE_PERCENT: u64 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.num_tokens > MAX_TOKEN_LIMIT.into() {
        return Err(ContractError::MaxTokenLimitExceeded {
            max: MAX_TOKEN_LIMIT,
        });
    }

    if let Some(per_address_limit) = msg.per_address_limit {
        // Check per address limit is valid
        if per_address_limit > MAX_PER_ADDRESS_LIMIT {
            return Err(ContractError::InvalidPerAddressLimit {
                max: MAX_PER_ADDRESS_LIMIT.to_string(),
                got: per_address_limit.to_string(),
            });
        }
    }

    if let Some(batch_mint_limit) = msg.batch_mint_limit {
        // Check batch mint limit is valid
        if batch_mint_limit > MAX_BATCH_MINT_LIMIT {
            return Err(ContractError::InvalidBatchMintLimit {
                max: MAX_BATCH_MINT_LIMIT.to_string(),
                got: batch_mint_limit.to_string(),
            });
        }
    }

    // Check that base_token_uri is a valid IPFS uri
    let parsed_token_uri = Url::parse(&msg.base_token_uri)?;
    if parsed_token_uri.scheme() != "ipfs" {
        return Err(ContractError::InvalidBaseTokenURI {});
    }

    if NATIVE_DENOM != msg.unit_price.denom {
        return Err(ContractError::InvalidDenom {
            expected: NATIVE_DENOM.to_string(),
            got: msg.unit_price.denom.to_string(),
        });
    }
    if MIN_MINT_PRICE > msg.unit_price.amount.into() {
        return Err(ContractError::InsufficientMintPrice {
            expected: MIN_MINT_PRICE,
            got: msg.unit_price.amount.into(),
        });
    }

    // Initially set batch_mint_limit if no msg
    let batch_mint_limit: Option<u64> = msg.batch_mint_limit.or(Some(STARTING_BATCH_MINT_LIMIT));

    // Initially set per_address_limit if no msg
    let per_address_limit: Option<u64> = msg.per_address_limit.or(Some(STARTING_PER_ADDRESS_LIMIT));

    let whitelist_addr: Option<Addr> = match msg.whitelist {
        Some(wl) => Some(deps.api.addr_validate(&wl)?),
        None => None,
    };

    // default is genesis mint start time
    let default_start_time = Expiration::AtTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME));
    let start_time = match msg.start_time {
        Some(st) => {
            if st < default_start_time {
                default_start_time
            } else {
                st
            }
        }
        None => default_start_time,
    };

    let config = Config {
        admin: info.sender.clone(),
        base_token_uri: msg.base_token_uri,
        num_tokens: msg.num_tokens,
        sg721_code_id: msg.sg721_code_id,
        unit_price: msg.unit_price,
        per_address_limit,
        batch_mint_limit,
        whitelist: whitelist_addr,
        start_time: Some(start_time),
    };
    CONFIG.save(deps.storage, &config)?;

    // save mintable token ids map
    for token_id in 1..=msg.num_tokens {
        MINTABLE_TOKEN_IDS.save(deps.storage, token_id, &Empty {})?;
    }

    let sub_msgs: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.sg721_code_id,
            msg: to_binary(&Sg721InstantiateMsg {
                name: msg.sg721_instantiate_msg.name,
                symbol: msg.sg721_instantiate_msg.symbol,
                minter: env.contract.address.to_string(),
                config: msg.sg721_instantiate_msg.config,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Fixed price minter"),
        }
        .into(),
        id: INSTANTIATE_SG721_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessages(sub_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => execute_mint_sender(deps, env, info),
        ExecuteMsg::UpdateStartTime(expiration) => {
            execute_update_start_time(deps, env, info, expiration)
        }
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::UpdateBatchMintLimit { batch_mint_limit } => {
            execute_update_batch_mint_limit(deps, env, info, batch_mint_limit)
        }
        ExecuteMsg::MintTo { recipient } => execute_mint_to(deps, env, info, recipient),
        ExecuteMsg::MintFor {
            token_id,
            recipient,
        } => execute_mint_for(deps, env, info, token_id, recipient),
        ExecuteMsg::BatchMint { num_mints } => execute_batch_mint(deps, env, info, num_mints),
        ExecuteMsg::SetWhitelist { whitelist } => {
            execute_set_whitelist(deps, env, info, &whitelist)
        }
    }
}

pub fn execute_set_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    whitelist: &str,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    };
    config.whitelist = Some(deps.api.addr_validate(whitelist)?);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "set_whitelist")
        .add_attribute("whitelist", whitelist.to_string()))
}

pub fn execute_mint_sender(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let action = "mint_sender";
    let mut pub_mint: bool = false;

    // check if a whitelist exists and not ended
    // sender has to be whitelisted to mint
    if let Some(whitelist) = config.whitelist {
        let res_started: HasStartedResponse = deps
            .querier
            .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::HasStarted {})?;
        let res_ended: HasEndedResponse = deps
            .querier
            .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::HasEnded {})?;
        if res_started.has_started && !res_ended.has_ended {
            let res: HasMemberResponse = deps.querier.query_wasm_smart(
                whitelist,
                &WhitelistQueryMsg::HasMember {
                    member: info.sender.to_string(),
                },
            )?;
            if !res.has_member {
                return Err(ContractError::NotWhitelisted {
                    addr: info.sender.to_string(),
                });
            }
        } else {
            pub_mint = true;
        }
    } else {
        pub_mint = true;
    }

    // if there is no active whitelist right now, check public mint
    if pub_mint {
        if let Some(start_time) = config.start_time {
            // Check if after start_time
            if !start_time.is_expired(&env.block) {
                return Err(ContractError::BeforeMintStartTime {});
            }
        }
    }

    // Check if already minted max per address limit
    if let Some(per_address_limit) = config.per_address_limit {
        let tokens: Cw721TokensResponse = deps.querier.query_wasm_smart(
            sg721_address.to_string(),
            &Sg721QueryMsg::Tokens {
                owner: info.sender.to_string(),
                start_after: None,
                limit: Some(MAX_PER_ADDRESS_LIMIT as u32),
            },
        )?;
        if tokens.tokens.len() >= per_address_limit as usize {
            return Err(ContractError::MaxPerAddressLimitExceeded {});
        }
    }

    _execute_mint(deps, env, info, action, false, None, None)
}

pub fn execute_mint_to(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let action = "mint_to";

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    _execute_mint(deps, env, info, action, true, Some(recipient), None)
}

pub fn execute_mint_for(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u64,
    recipient: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let action = "mint_for";

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    _execute_mint(
        deps,
        env,
        info,
        action,
        true,
        Some(recipient),
        Some(token_id),
    )
}

pub fn execute_batch_mint(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    num_mints: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mint_limit = config
        .batch_mint_limit
        .ok_or(ContractError::MaxBatchMintLimitExceeded {})?;

    if num_mints > mint_limit {
        return Err(ContractError::MaxBatchMintLimitExceeded {});
    }

    // NOTE: fees are handled in the `_execute_mint` function

    let mut msgs: Vec<CosmosMsg<StargazeMsgWrapper>> = vec![];
    let mint_msg = ExecuteMsg::Mint {};
    let msg: CosmosMsg<StargazeMsgWrapper> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![mint_price(deps.as_ref(), false)?],
    });
    for _ in 0..num_mints {
        msgs.append(&mut vec![msg.clone()]);
    }

    Ok(Response::default()
        .add_attribute("action", "batch_mint")
        .add_attribute("num_mints", num_mints.to_string())
        .add_messages(msgs))
}

fn _execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: &str,
    admin_no_fee: bool,
    recipient: Option<Addr>,
    token_id: Option<u64>,
) -> Result<Response, ContractError> {
    // generalize checks and mint message creation
    // mint -> _execute_mint(recipient: None, token_id: None)
    // mint_to(recipient: "friend") -> _execute_mint(Some(recipient), token_id: None)
    // mint_for(recipient: "friend2", token_id: 420) -> _execute_mint(recipient, token_id)
    let mut msgs: Vec<CosmosMsg<StargazeMsgWrapper>> = vec![];
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let recipient_addr = if recipient.is_none() {
        info.sender.clone()
    } else if let Some(some_recipient) = recipient {
        some_recipient
    } else {
        return Err(ContractError::InvalidAddress {
            addr: info.sender.to_string(),
        });
    };

    let mint_price: Coin = mint_price(deps.as_ref(), admin_no_fee)?;
    // exact payment only
    let payment = may_pay(&info, &config.unit_price.denom)?;
    if payment != mint_price.amount {
        return Err(ContractError::IncorrectPaymentAmount(
            coin(payment.u128(), &config.unit_price.denom),
            mint_price,
        ));
    }

    // guardrail against low mint price updates
    if MIN_MINT_PRICE > mint_price.amount.into() && !admin_no_fee {
        return Err(ContractError::InsufficientMintPrice {
            expected: MIN_MINT_PRICE,
            got: mint_price.amount.into(),
        });
    }

    // create network fee msgs
    let network_fee: Uint128 = if admin_no_fee {
        Uint128::zero()
    } else {
        let fee_percent = Decimal::percent(MINT_FEE_PERCENT);
        let network_fee = mint_price.amount * fee_percent;
        msgs.append(&mut burn_and_distribute_fee(
            env,
            &info,
            network_fee.u128(),
        )?);
        network_fee
    };

    // if token_id None, find and assign one. else check token_id exists on mintable map.
    let mintable_token_id: u64 = if token_id.is_none() {
        let mintable_tokens_result: StdResult<Vec<u64>> = MINTABLE_TOKEN_IDS
            .keys(deps.storage, None, None, Order::Ascending)
            .take(1)
            .collect();
        let mintable_tokens = mintable_tokens_result?;
        if mintable_tokens.is_empty() {
            return Err(ContractError::SoldOut {});
        }
        mintable_tokens[0]
    } else if let Some(some_token_id) = token_id {
        // If token_id not on mintable map, throw err
        if !MINTABLE_TOKEN_IDS.has(deps.storage, some_token_id) {
            return Err(ContractError::TokenIdAlreadySold {
                token_id: some_token_id,
            });
        }
        some_token_id
    } else {
        return Err(ContractError::InvalidTokenId {});
    };

    // create mint msgs
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Empty> {
        token_id: mintable_token_id.to_string(),
        owner: recipient_addr.to_string(),
        token_uri: Some(format!("{}/{}", config.base_token_uri, mintable_token_id)),
        extension: Empty {},
    });
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sg721_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    msgs.append(&mut vec![msg]);

    // remove mintable token id from map
    MINTABLE_TOKEN_IDS.remove(deps.storage, mintable_token_id);

    let mut seller_amount = Uint128::zero();
    // does have a fee
    if !admin_no_fee {
        seller_amount = mint_price.amount - network_fee;
        msgs.append(&mut vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: config.admin.to_string(),
            amount: vec![coin(seller_amount.u128(), config.unit_price.denom)],
        })]);
    };

    Ok(Response::default()
        .add_attribute("action", action)
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("network_fee", network_fee)
        .add_attribute("mint_price", mint_price.amount)
        .add_attribute("seller_amount", seller_amount)
        .add_attribute("no_fee", admin_no_fee.to_string())
        .add_messages(msgs))
}

pub fn execute_update_start_time(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    start_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    let default_start_time = Expiration::AtTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME));
    let start_time = if start_time < default_start_time {
        default_start_time
    } else {
        start_time
    };

    config.start_time = Some(start_time);
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("sender", info.sender)
        .add_attribute("start_time", start_time.to_string()))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    per_address_limit: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }
    if per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT.to_string(),
            got: per_address_limit.to_string(),
        });
    }
    config.per_address_limit = Some(per_address_limit);
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_per_address_limit")
        .add_attribute("sender", info.sender)
        .add_attribute("limit", per_address_limit.to_string()))
}

pub fn execute_update_batch_mint_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    batch_mint_limit: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }
    if batch_mint_limit > MAX_BATCH_MINT_LIMIT {
        return Err(ContractError::InvalidBatchMintLimit {
            max: MAX_BATCH_MINT_LIMIT.to_string(),
            got: batch_mint_limit.to_string(),
        });
    }
    config.batch_mint_limit = Some(batch_mint_limit);
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_batch_mint_limit")
        .add_attribute("limit", batch_mint_limit.to_string()))
}

pub fn mint_price(deps: Deps, admin_no_fee: bool) -> Result<Coin, StdError> {
    // if admin => admin mint fee,
    // else if in whitelist => whitelist price
    // else => config unit price
    let config = CONFIG.load(deps.storage)?;
    let mint_price: Coin = if admin_no_fee {
        Coin {
            amount: Uint128::zero(),
            denom: NATIVE_DENOM.to_string(),
        }
    } else if let Some(whitelist) = config.whitelist {
        let res_started: HasStartedResponse = deps
            .querier
            .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::HasStarted {})?;
        let res_ended: HasEndedResponse = deps
            .querier
            .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::HasEnded {})?;
        if res_started.has_started && !res_ended.has_ended {
            let unit_price: UnitPriceResponse = deps
                .querier
                .query_wasm_smart(whitelist, &WhitelistQueryMsg::UnitPrice {})?;
            unit_price.unit_price
        } else {
            config.unit_price.clone()
        }
    } else {
        config.unit_price.clone()
    };
    Ok(mint_price)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::StartTime {} => to_binary(&query_start_time(deps)?),
        QueryMsg::MintableNumTokens {} => to_binary(&query_mintable_num_tokens(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        base_token_uri: config.base_token_uri,
        sg721_address,
        sg721_code_id: config.sg721_code_id,
        num_tokens: config.num_tokens,
        start_time: config.start_time,
        unit_price: config.unit_price,
        per_address_limit: config.per_address_limit,
        batch_mint_limit: config.batch_mint_limit,
        whitelist: config.whitelist,
    })
}

fn query_start_time(deps: Deps) -> StdResult<StartTimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    if let Some(expiration) = config.start_time {
        Ok(StartTimeResponse {
            start_time: expiration.to_string(),
        })
    } else {
        Err(StdError::GenericErr {
            msg: "start time not found".to_string(),
        })
    }
}

fn query_mintable_num_tokens(deps: Deps) -> StdResult<MintableNumTokensResponse> {
    let count = MINTABLE_TOKEN_IDS
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    Ok(MintableNumTokensResponse {
        count: count as u64,
    })
}
// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_SG721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            SG721_ADDRESS.save(deps.storage, &Addr::unchecked(res.contract_address))?;
            Ok(Response::default().add_attribute("action", "instantiate_sg721_reply"))
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}
