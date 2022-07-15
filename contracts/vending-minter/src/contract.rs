use std::convert::TryInto;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, MintCountResponse, MintPriceResponse, MintableNumTokensResponse,
    MintableTokensResponse, QueryMsg, StartTimeResponse,
};
use crate::state::{
    Config, CONFIG, MINTABLE_NUM_TOKENS, MINTABLE_TOKEN_POSITIONS, MINTER_ADDRS, SG721_ADDRESS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, ReplyOn, StdError, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_utils::{may_pay, parse_reply_instantiate_data};
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sg1::checked_fair_burn;
use sg721::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg_std::math::U64Ext;
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::msg::{
    ConfigResponse as WhitelistConfigResponse, HasMemberResponse, QueryMsg as WhitelistQueryMsg,
};
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};
use url::Url;

use vending::{
    ParamsResponse, QueryMsg as LaunchpadQueryMsg, VendingMinterCreateMsg as InstantiateMsg,
};

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

pub struct TokenPositionMapping {
    pub position: u32,
    pub token_id: u32,
}

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_SG721_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let factory = info.sender.clone();

    // TODO: validate

    // Make sure the sender is the factory contract
    // This will fail if the sender cannot parse a response from the factory contract
    let _res: ParamsResponse = deps
        .querier
        .query_wasm_smart(factory.clone(), &LaunchpadQueryMsg::Params {})?;

    // Check that base_token_uri is a valid IPFS uri
    let parsed_token_uri = Url::parse(&msg.init_msg.base_token_uri)?;
    if parsed_token_uri.scheme() != "ipfs" {
        return Err(ContractError::InvalidBaseTokenURI {});
    }

    let genesis_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    // If start time is before genesis time return error
    if msg.init_msg.start_time < genesis_time {
        return Err(ContractError::BeforeGenesisTime {});
    }

    // If current time is beyond the provided start time return error
    if env.block.time > msg.init_msg.start_time {
        return Err(ContractError::InvalidStartTime(
            msg.init_msg.start_time,
            env.block.time,
        ));
    }

    // Validate address for the optional whitelist contract
    let whitelist_addr = msg
        .init_msg
        .whitelist
        .and_then(|w| deps.api.addr_validate(w.as_str()).ok());

    let config = Config {
        admin: deps
            .api
            .addr_validate(&msg.collection_params.info.creator)?,
        factory: factory.clone(),
        base_token_uri: msg.init_msg.base_token_uri,
        num_tokens: msg.init_msg.num_tokens,
        sg721_code_id: msg.collection_params.code_id,
        unit_price: msg.init_msg.unit_price,
        per_address_limit: msg.init_msg.per_address_limit,
        whitelist: whitelist_addr,
        start_time: msg.init_msg.start_time,
    };
    CONFIG.save(deps.storage, &config)?;
    MINTABLE_NUM_TOKENS.save(deps.storage, &msg.init_msg.num_tokens)?;

    let token_ids = random_token_list(
        &env,
        deps.api
            .addr_validate(&msg.collection_params.info.creator)?,
        (1..=msg.init_msg.num_tokens).collect::<Vec<u32>>(),
    )?;
    // Save mintable token ids map
    let mut token_position = 1;
    for token_id in token_ids {
        MINTABLE_TOKEN_POSITIONS.save(deps.storage, token_position, &token_id)?;
        token_position += 1;
    }

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
            admin: Some(config.admin.to_string()),
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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => execute_mint_sender(deps, env, info),
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::MintTo { recipient } => execute_mint_to(deps, env, info, recipient),
        ExecuteMsg::MintFor {
            token_id,
            recipient,
        } => execute_mint_for(deps, env, info, token_id, recipient),
        ExecuteMsg::SetWhitelist { whitelist } => {
            execute_set_whitelist(deps, env, info, &whitelist)
        }
        ExecuteMsg::Shuffle {} => execute_shuffle(deps, env, info),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
    }
}

// Anyone can pay to shuffle at any time
// Introduces another source of randomness to minting
// There's a fee because this action is expensive.
pub fn execute_shuffle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut res = Response::new();

    let config = CONFIG.load(deps.storage)?;

    let factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &LaunchpadQueryMsg::Params {})?;

    // Check exact shuffle fee payment included in message
    checked_fair_burn(
        &info,
        factory_params.params.extension.shuffle_fee.amount.u128(),
        None,
        &mut res,
    )?;

    // Check not sold out
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    if mintable_num_tokens == 0 {
        return Err(ContractError::SoldOut {});
    }

    // get positions and token_ids, then randomize token_ids and reassign positions
    let mut positions = vec![];
    let mut token_ids = vec![];
    for mapping in MINTABLE_TOKEN_POSITIONS.range(deps.storage, None, None, Order::Ascending) {
        let (position, token_id) = mapping?;
        positions.push(position);
        token_ids.push(token_id);
    }
    let randomized_token_ids = random_token_list(&env, info.sender.clone(), token_ids.clone())?;
    for (i, position) in positions.iter().enumerate() {
        MINTABLE_TOKEN_POSITIONS.save(deps.storage, *position, &randomized_token_ids[i])?;
    }

    Ok(res
        .add_attribute("action", "shuffle")
        .add_attribute("sender", info.sender))
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    };

    // query balance from the contract
    let balance = deps
        .querier
        .query_balance(env.contract.address, NATIVE_DENOM)?;
    if balance.amount.is_zero() {
        return Err(ContractError::NotEnoughFunds {});
    }

    // send contract balance to creator
    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![balance],
    });

    Ok(Response::default()
        .add_attribute("action", "withdraw")
        .add_message(send_msg))
}

pub fn execute_set_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelist: &str,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    };

    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if let Some(wl) = config.whitelist {
        let res: WhitelistConfigResponse = deps
            .querier
            .query_wasm_smart(wl, &WhitelistQueryMsg::Config {})?;

        if res.is_active {
            return Err(ContractError::WhitelistAlreadyStarted {});
        }
    }

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
    let action = "mint_sender";

    // If there is no active whitelist right now, check public mint
    // Check if after start_time
    if is_public_mint(deps.as_ref(), &info)? && (env.block.time < config.start_time) {
        return Err(ContractError::BeforeMintStartTime {});
    }

    // Check if already minted max per address limit
    let mint_count = mint_count(deps.as_ref(), &info)?;
    if mint_count >= config.per_address_limit {
        return Err(ContractError::MaxPerAddressLimitExceeded {});
    }

    _execute_mint(deps, env, info, action, false, None, None)
}

// Check if a whitelist exists and not ended
// Sender has to be whitelisted to mint
fn is_public_mint(deps: Deps, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // If there is no whitelist, there's only a public mint
    if config.whitelist.is_none() {
        return Ok(true);
    }

    let whitelist = config.whitelist.unwrap();

    let wl_config: WhitelistConfigResponse = deps
        .querier
        .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::Config {})?;

    if !wl_config.is_active {
        return Ok(true);
    }

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

    // Check wl per address limit
    let mint_count = mint_count(deps, info)?;
    if mint_count >= wl_config.per_address_limit {
        return Err(ContractError::MaxPerAddressLimitExceeded {});
    }

    Ok(false)
}

pub fn execute_mint_to(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;
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
    token_id: u32,
    recipient: String,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;
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

// Generalize checks and mint message creation
// mint -> _execute_mint(recipient: None, token_id: None)
// mint_to(recipient: "friend") -> _execute_mint(Some(recipient), token_id: None)
// mint_for(recipient: "friend2", token_id: 420) -> _execute_mint(recipient, token_id)
fn _execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: &str,
    is_admin: bool,
    recipient: Option<Addr>,
    token_id: Option<u32>,
) -> Result<Response, ContractError> {
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    if mintable_num_tokens == 0 {
        return Err(ContractError::SoldOut {});
    }

    let config = CONFIG.load(deps.storage)?;

    if let Some(token_id) = token_id {
        if token_id == 0 || token_id > config.num_tokens {
            return Err(ContractError::InvalidTokenId {});
        }
    }

    let sg721_address = SG721_ADDRESS.load(deps.storage)?;

    let recipient_addr = match recipient {
        Some(some_recipient) => some_recipient,
        None => info.sender.clone(),
    };

    let mint_price: Coin = mint_price(deps.as_ref(), is_admin)?;
    // Exact payment only accepted
    let payment = may_pay(&info, &config.unit_price.denom)?;
    if payment != mint_price.amount {
        return Err(ContractError::IncorrectPaymentAmount(
            coin(payment.u128(), &config.unit_price.denom),
            mint_price,
        ));
    }

    let mut res = Response::new();

    let factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &LaunchpadQueryMsg::Params {})?;

    // Create network fee msgs
    let mint_fee = if is_admin {
        factory_params.params.airdrop_mint_fee_bps.bps_to_decimal()
    } else {
        factory_params.params.mint_fee_bps.bps_to_decimal()
    };
    let network_fee = mint_price.amount * mint_fee;
    checked_fair_burn(&info, network_fee.u128(), None, &mut res)?;

    let mintable_token_mapping = match token_id {
        Some(token_id) => {
            // set position to invalid value, iterate to find matching token_id
            // if token_id not found, token_id is already sold, position is unchanged and throw err
            // otherwise return position and token_id
            let mut position = 0;
            for res in MINTABLE_TOKEN_POSITIONS.range(deps.storage, None, None, Order::Ascending) {
                let (pos, id) = res?;
                if id == token_id {
                    position = pos;
                    break;
                }
            }
            if position == 0 {
                return Err(ContractError::TokenIdAlreadySold { token_id });
            }
            TokenPositionMapping { position, token_id }
        }
        None => random_mintable_token_mapping(deps.as_ref(), env, info.sender.clone())?,
    };

    // Create mint msgs
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Empty> {
        token_id: mintable_token_mapping.token_id.to_string(),
        owner: recipient_addr.to_string(),
        token_uri: Some(format!(
            "{}/{}",
            config.base_token_uri, mintable_token_mapping.token_id
        )),
        extension: Empty {},
    });
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sg721_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    res = res.add_message(msg);

    // Remove mintable token position from map
    MINTABLE_TOKEN_POSITIONS.remove(deps.storage, mintable_token_mapping.position);
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    // Decrement mintable num tokens
    MINTABLE_NUM_TOKENS.save(deps.storage, &(mintable_num_tokens - 1))?;
    // Save the new mint count for the sender's address
    let new_mint_count = mint_count(deps.as_ref(), &info)? + 1;
    MINTER_ADDRS.save(deps.storage, info.clone().sender, &new_mint_count)?;

    let seller_amount = if !is_admin {
        let amount = mint_price.amount - network_fee;
        let msg = BankMsg::Send {
            to_address: config.admin.to_string(),
            amount: vec![coin(amount.u128(), config.unit_price.denom)],
        };
        res = res.add_message(msg);
        amount
    } else {
        Uint128::zero()
    };

    Ok(res
        .add_attribute("action", action)
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("token_id", mintable_token_mapping.token_id.to_string())
        .add_attribute("network_fee", network_fee)
        .add_attribute("mint_price", mint_price.amount)
        .add_attribute("seller_amount", seller_amount))
}

fn random_token_list(
    env: &Env,
    sender: Addr,
    mut tokens: Vec<u32>,
) -> Result<Vec<u32>, ContractError> {
    let sha256 =
        Sha256::digest(format!("{}{}{}", sender, env.block.height, tokens.len()).into_bytes());
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    let mut shuffler = FisherYates::default();
    shuffler
        .shuffle(&mut tokens, &mut rng)
        .map_err(StdError::generic_err)?;
    Ok(tokens)
}

// Does a baby shuffle, picking a token_id from the first or last 50 mintable positions.
fn random_mintable_token_mapping(
    deps: Deps,
    env: Env,
    sender: Addr,
) -> Result<TokenPositionMapping, ContractError> {
    let num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    let sha256 =
        Sha256::digest(format!("{}{}{}", sender, num_tokens, env.block.height).into_bytes());
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();

    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);

    let r = rng.next_u32();

    let order = match r % 2 {
        1 => Order::Descending,
        _ => Order::Ascending,
    };
    let mut rem = 50;
    if rem > num_tokens {
        rem = num_tokens;
    }
    let n = r % rem;
    let position = MINTABLE_TOKEN_POSITIONS
        .keys(deps.storage, None, None, order)
        .skip(n as usize)
        .take(1)
        .collect::<StdResult<Vec<_>>>()?[0];

    let token_id = MINTABLE_TOKEN_POSITIONS.load(deps.storage, position)?;
    Ok(TokenPositionMapping { position, token_id })
}

pub fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }
    // If current time is after the stored start time return error
    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    // If current time already passed the new start_time return error
    if env.block.time > start_time {
        return Err(ContractError::InvalidStartTime(start_time, env.block.time));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    // If the new start_time is before genesis start time return error
    if start_time < genesis_start_time {
        return Err(ContractError::BeforeGenesisTime {});
    }

    config.start_time = start_time;
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
    per_address_limit: u32,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    let factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory.clone(), &LaunchpadQueryMsg::Params {})?;

    if per_address_limit == 0 || per_address_limit > factory_params.params.max_per_address_limit {
        return Err(ContractError::InvalidPerAddressLimit {
            max: factory_params.params.max_per_address_limit,
            min: 1,
            got: per_address_limit,
        });
    }
    config.per_address_limit = per_address_limit;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_per_address_limit")
        .add_attribute("sender", info.sender)
        .add_attribute("limit", per_address_limit.to_string()))
}

// if admin_no_fee => no fee,
// else if in whitelist => whitelist price
// else => config unit price
pub fn mint_price(deps: Deps, is_admin: bool) -> Result<Coin, StdError> {
    let config = CONFIG.load(deps.storage)?;

    let factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &LaunchpadQueryMsg::Params {})?;

    if is_admin {
        return Ok(coin(
            factory_params.params.airdrop_mint_price.amount.u128(),
            config.unit_price.denom,
        ));
    }

    if config.whitelist.is_none() {
        return Ok(config.unit_price);
    }

    let whitelist = config.whitelist.unwrap();

    let wl_config: WhitelistConfigResponse = deps
        .querier
        .query_wasm_smart(whitelist, &WhitelistQueryMsg::Config {})?;

    if wl_config.is_active {
        Ok(wl_config.unit_price)
    } else {
        Ok(config.unit_price)
    }
}

fn mint_count(deps: Deps, info: &MessageInfo) -> Result<u32, StdError> {
    let mint_count = (MINTER_ADDRS
        .key(info.sender.clone())
        .may_load(deps.storage)?)
    .unwrap_or(0);
    Ok(mint_count)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::StartTime {} => to_binary(&query_start_time(deps)?),
        QueryMsg::MintableNumTokens {} => to_binary(&query_mintable_num_tokens(deps)?),
        QueryMsg::MintPrice {} => to_binary(&query_mint_price(deps)?),
        QueryMsg::MintCount { address } => to_binary(&query_mint_count(deps, address)?),
        // TODO for debug to test shuffle. remove before prod
        QueryMsg::MintableTokens {} => to_binary(&query_mintable_tokens(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        base_token_uri: config.base_token_uri,
        sg721_address: sg721_address.to_string(),
        sg721_code_id: config.sg721_code_id,
        num_tokens: config.num_tokens,
        start_time: config.start_time,
        unit_price: config.unit_price,
        per_address_limit: config.per_address_limit,
        whitelist: config.whitelist.map(|w| w.to_string()),
        factory: config.factory.to_string(),
    })
}

fn query_mint_count(deps: Deps, address: String) -> StdResult<MintCountResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let mint_count = (MINTER_ADDRS.key(addr.clone()).may_load(deps.storage)?).unwrap_or(0);
    Ok(MintCountResponse {
        address: addr.to_string(),
        count: mint_count,
    })
}

fn query_start_time(deps: Deps) -> StdResult<StartTimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(StartTimeResponse {
        start_time: config.start_time.to_string(),
    })
}

fn query_mintable_num_tokens(deps: Deps) -> StdResult<MintableNumTokensResponse> {
    let count = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    Ok(MintableNumTokensResponse { count })
}

//TODO for debug to test shuffle. remove before prod
fn query_mintable_tokens(deps: Deps) -> StdResult<MintableTokensResponse> {
    let tokens = MINTABLE_TOKEN_POSITIONS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|t| t.unwrap())
        .collect::<Vec<_>>();

    Ok(MintableTokensResponse {
        mintable_tokens: tokens,
    })
}

fn query_mint_price(deps: Deps) -> StdResult<MintPriceResponse> {
    let config = CONFIG.load(deps.storage)?;

    let current_price = mint_price(deps, false)?;
    let public_price = config.unit_price;
    let whitelist_price: Option<Coin> = if let Some(whitelist) = config.whitelist {
        let wl_config: WhitelistConfigResponse = deps
            .querier
            .query_wasm_smart(whitelist, &WhitelistQueryMsg::Config {})?;
        Some(wl_config.unit_price)
    } else {
        None
    };
    Ok(MintPriceResponse {
        current_price,
        public_price,
        whitelist_price,
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
            let sg721_address = res.contract_address;

            // mark the collection contract as ready to mint
            let msg = WasmMsg::Execute {
                contract_addr: sg721_address.clone(),
                msg: to_binary(&Sg721ExecuteMsg::<Empty>::_Ready {})?,
                funds: vec![],
            };

            SG721_ADDRESS.save(deps.storage, &Addr::unchecked(sg721_address))?;

            Ok(Response::default()
                .add_attribute("action", "instantiate_sg721_reply")
                .add_message(msg))
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}
