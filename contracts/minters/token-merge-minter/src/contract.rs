use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, MintCountResponse, MintTokensResponse, MintableNumTokensResponse,
    QueryMsg, ReceiveNftMsg, StartTimeResponse,
};
use crate::state::{
    Config, ConfigExtension, CONFIG, MINTABLE_NUM_TOKENS, MINTABLE_TOKEN_POSITIONS, MINTER_ADDRS,
    RECEIVED_TOKENS, SG721_ADDRESS, STATUS,
};
use crate::validation::{check_dynamic_per_address_limit, get_three_percent_of_tokens};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure, from_json, to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Empty, Env, Event, MessageInfo, Order, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;
use cw721_base::Extension;
use cw_utils::{may_pay, nonpayable, parse_reply_instantiate_data};
use nois::{int_in_range, shuffle};

use semver::Version;
use sg1::{checked_fair_burn, distribute_mint_fees};
use sg4::{Status, StatusResponse, SudoMsg};
use sg721::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg_utils::GENESIS_MINT_START_TIME;
use sha2::{Digest, Sha256};

use std::convert::TryInto;
use token_merge_factory::msg::QueryMsg as FactoryQueryMsg;
use token_merge_factory::msg::{MintToken, ParamsResponse, TokenMergeMinterCreateMsg};
use url::Url;

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
    msg: TokenMergeMinterCreateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let factory = info.sender.clone();

    // Make sure the sender is the factory contract
    // This will fail if the sender cannot parse a response from the factory contract
    let factory_response: ParamsResponse = deps
        .querier
        .query_wasm_smart(factory.clone(), &FactoryQueryMsg::Params {})?;
    let factory_params = factory_response.params;

    // set default status so it can be queried without failing
    STATUS.save(deps.storage, &Status::default())?;

    if !check_dynamic_per_address_limit(
        msg.init_msg.per_address_limit,
        msg.init_msg.num_tokens,
        factory_params.max_per_address_limit,
    )? {
        return Err(ContractError::InvalidPerAddressLimit {
            max: display_max_mintable_tokens(
                msg.init_msg.per_address_limit,
                msg.init_msg.num_tokens,
                factory_params.max_per_address_limit,
            )?,
            min: 1,
            got: msg.init_msg.per_address_limit,
        });
    }

    // sanitize base token uri
    let mut base_token_uri = msg.init_msg.base_token_uri.trim().to_string();
    // Token URI must be a valid URL (ipfs, https, etc.)
    let parsed_token_uri =
        Url::parse(&base_token_uri).map_err(|_| ContractError::InvalidBaseTokenURI {})?;
    base_token_uri = parsed_token_uri.to_string();

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

    // Use default start trading time if not provided
    let mut collection_info = msg.collection_params.info.clone();
    let offset = factory_params.max_trading_offset_secs;
    let default_start_time_with_offset = msg.init_msg.start_time.plus_seconds(offset);
    if let Some(start_trading_time) = msg.collection_params.info.start_trading_time {
        // If trading start time > start_time + offset, return error
        if start_trading_time > default_start_time_with_offset {
            return Err(ContractError::InvalidStartTradingTime(
                start_trading_time,
                default_start_time_with_offset,
            ));
        }
    }
    let start_trading_time = msg
        .collection_params
        .info
        .start_trading_time
        .or(Some(default_start_time_with_offset));
    collection_info.start_trading_time = start_trading_time;

    let config = Config {
        factory: factory.clone(),
        collection_code_id: msg.collection_params.code_id,
        extension: ConfigExtension {
            admin: deps
                .api
                .addr_validate(&msg.collection_params.info.creator)?,
            base_token_uri,
            num_tokens: msg.init_msg.num_tokens,
            per_address_limit: msg.init_msg.per_address_limit,
            start_time: msg.init_msg.start_time,
            mint_tokens: msg.init_msg.mint_tokens,
        },
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
            msg: to_json_binary(&Sg721InstantiateMsg {
                name: msg.collection_params.name.clone(),
                symbol: msg.collection_params.symbol,
                minter: env.contract.address.to_string(),
                collection_info,
            })?,
            funds: info.funds,
            admin: Some(config.extension.admin.to_string()),
            label: format!("SG721-{}", msg.collection_params.name.trim()),
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
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender,
            token_id,
            msg: raw_msg,
        }) => {
            let msg: ReceiveNftMsg = from_json(raw_msg)?;
            match msg {
                ReceiveNftMsg::DepositToken { recipient } => {
                    execute_receive_nft(deps, env, info, sender, token_id, recipient)
                }
            }
        }
        ExecuteMsg::Purge {} => execute_purge(deps, env, info),
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdateStartTradingTime(time) => {
            execute_update_start_trading_time(deps, env, info, time)
        }
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::MintTo { recipient } => execute_mint_to(deps, env, info, recipient),
        ExecuteMsg::MintFor {
            token_id,
            recipient,
        } => execute_mint_for(deps, env, info, token_id, recipient),
        ExecuteMsg::Shuffle {} => execute_shuffle(deps, env, info),
        ExecuteMsg::BurnRemaining {} => execute_burn_remaining(deps, env, info),
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    token_id: String,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut action = "receive_and_burn_nft";
    ensure!(
        env.block.time > config.extension.start_time,
        ContractError::BeforeMintStartTime {}
    );

    let recipient_addr = deps
        .api
        .addr_validate(&recipient.unwrap_or(sender.clone()))?;

    let mint_count = mint_count(deps.as_ref(), recipient_addr.clone())?;
    if mint_count >= config.extension.per_address_limit {
        return Err(ContractError::MaxPerAddressLimitExceeded {});
    }

    // Check received token is from an expected collection
    let valid_mint_token = config
        .extension
        .mint_tokens
        .iter()
        .find(|token| token.collection == info.sender);
    ensure!(
        valid_mint_token.is_some(),
        ContractError::InvalidCollection {}
    );

    let already_received_amount = RECEIVED_TOKENS
        .load(deps.storage, (&recipient_addr, info.sender.to_string()))
        .unwrap_or(0);
    ensure!(
        already_received_amount < valid_mint_token.unwrap().amount,
        ContractError::TooManyTokens {}
    );
    RECEIVED_TOKENS.save(
        deps.storage,
        (&recipient_addr, info.sender.to_string()),
        &(already_received_amount + 1),
    )?;

    let mint_requirement_fulfilled = check_all_mint_tokens_received(
        deps.as_ref(),
        recipient_addr.clone(),
        config.extension.mint_tokens,
    )?;

    // Create the burn message for the received token
    let burn_msg = Sg721ExecuteMsg::<Extension, Empty>::Burn {
        token_id: token_id.clone(),
    };
    let burn_cosmos_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.sender.to_string(),
        msg: to_json_binary(&burn_msg)?,
        funds: vec![],
    });

    if !mint_requirement_fulfilled {
        return Ok(Response::new()
            .add_message(burn_cosmos_msg)
            .add_attribute("action", action)
            .add_attribute("sender", sender)
            .add_attribute("collection", info.sender.to_string())
            .add_attribute("token_id", token_id));
    }

    action = "mint_sender";
    _execute_mint(
        deps,
        env,
        info,
        action,
        false,
        Some(recipient_addr),
        None,
        Some(burn_cosmos_msg),
    )
}

// Purge frees data after a mint is sold out
// Anyone can purge
pub fn execute_purge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    // check mint sold out
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    if mintable_num_tokens != 0 {
        return Err(ContractError::NotSoldOut {});
    }

    let keys = MINTER_ADDRS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Vec<_>>();
    for key in keys {
        MINTER_ADDRS.remove(deps.storage, &key?);
    }

    Ok(Response::new()
        .add_attribute("action", "purge")
        .add_attribute("contract", env.contract.address.to_string())
        .add_attribute("sender", info.sender))
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

    let factory: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &FactoryQueryMsg::Params {})?;
    let factory_params = factory.params;

    // Check exact shuffle fee payment included in message
    checked_fair_burn(
        &info,
        &env,
        factory_params.shuffle_fee.amount.u128(),
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
    if info.sender != config.extension.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    _execute_mint(deps, env, info, action, true, Some(recipient), None, None)
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
    if info.sender != config.extension.admin {
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
        None,
    )
}

fn check_all_mint_tokens_received(
    deps: Deps,
    sender: Addr,
    mint_tokens: Vec<MintToken>,
) -> Result<bool, ContractError> {
    for mint_token in mint_tokens {
        let received_amount = RECEIVED_TOKENS
            .load(deps.storage, (&sender, mint_token.collection.clone()))
            .unwrap_or(0);
        if received_amount < mint_token.amount {
            return Ok(false);
        }
    }
    Ok(true)
}

// Generalize checks and mint message creation
// ReceiveNFT -> _execute_mint(recipient: None, token_id: None)
// mint_to(recipient: "friend") -> _execute_mint(Some(recipient), token_id: None)
// mint_for(recipient: "friend2", token_id: 420) -> _execute_mint(recipient, token_id)
#[allow(clippy::too_many_arguments)]
fn _execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: &str,
    is_admin: bool,
    recipient: Option<Addr>,
    token_id: Option<u32>,
    burn_message: Option<CosmosMsg>,
) -> Result<Response, ContractError> {
    let mut network_fee = Uint128::zero();
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    if mintable_num_tokens == 0 {
        return Err(ContractError::SoldOut {});
    }

    let config = CONFIG.load(deps.storage)?;

    if let Some(token_id) = token_id {
        if token_id == 0 || token_id > config.extension.num_tokens {
            return Err(ContractError::InvalidTokenId {});
        }
    }

    let sg721_address = SG721_ADDRESS.load(deps.storage)?;

    let recipient_addr = match recipient {
        Some(some_recipient) => some_recipient,
        None => info.sender.clone(),
    };

    let factory: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory, &FactoryQueryMsg::Params {})?;
    let factory_params = factory.params;

    if is_admin {
        let airdrop_price: Coin = coin(
            factory_params.airdrop_mint_price.amount.u128(),
            factory_params.airdrop_mint_price.denom.clone(),
        );

        // Exact payment only accepted
        let payment = may_pay(&info, &airdrop_price.denom)?;
        if payment != airdrop_price.amount {
            return Err(ContractError::IncorrectPaymentAmount(
                coin(
                    payment.u128(),
                    factory_params.airdrop_mint_price.denom.clone(),
                ),
                factory_params.airdrop_mint_price,
            ));
        }
        let airdrop_fee_bps = Decimal::bps(factory_params.airdrop_mint_fee_bps);
        network_fee = airdrop_price.amount * airdrop_fee_bps;
    }

    let mut res = Response::new();

    if !network_fee.is_zero() {
        distribute_mint_fees(
            coin(
                network_fee.u128(),
                factory_params.airdrop_mint_price.clone().denom,
            ),
            &mut res,
            false,
            None,
        )?;
    }

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
    let mint_msg = Sg721ExecuteMsg::<Extension, Empty>::Mint {
        token_id: mintable_token_mapping.token_id.to_string(),
        owner: recipient_addr.to_string(),
        token_uri: Some(format!(
            "{}/{}",
            config.extension.base_token_uri, mintable_token_mapping.token_id
        )),
        extension: None,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sg721_address.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    });
    res = res.add_message(msg);

    // Burn the final token received
    if let Some(burn_message) = burn_message {
        res = res.add_message(burn_message);
        // Clear received tokens record for recipient
        for mint_token in config.extension.mint_tokens.iter() {
            RECEIVED_TOKENS.remove(
                deps.storage,
                (&recipient_addr, mint_token.collection.clone()),
            );
        }
    }

    // Remove mintable token position from map
    MINTABLE_TOKEN_POSITIONS.remove(deps.storage, mintable_token_mapping.position);
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    // Decrement mintable num tokens
    MINTABLE_NUM_TOKENS.save(deps.storage, &(mintable_num_tokens - 1))?;
    // Save the new mint count for the recipient's address
    let new_mint_count = mint_count(deps.as_ref(), recipient_addr.clone())? + 1;
    MINTER_ADDRS.save(deps.storage, &recipient_addr, &new_mint_count)?;

    Ok(res
        .add_attribute("action", action)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("token_id", mintable_token_mapping.token_id.to_string())
        .add_attribute(
            "network_fee",
            coin(network_fee.u128(), factory_params.airdrop_mint_price.denom).to_string(),
        ))
}

fn random_token_list(
    env: &Env,
    sender: Addr,
    mut tokens: Vec<u32>,
) -> Result<Vec<u32>, ContractError> {
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };
    let sha256 = Sha256::digest(
        format!("{}{}{}{}", sender, env.block.height, tokens.len(), tx_index).into_bytes(),
    );
    let randomness: [u8; 32] = sha256.to_vec().try_into().unwrap();
    tokens = shuffle(randomness, tokens);
    Ok(tokens)
}

// Does a baby shuffle, picking a token_id from the first or last 50 mintable positions.
fn random_mintable_token_mapping(
    deps: Deps,
    env: Env,
    sender: Addr,
) -> Result<TokenPositionMapping, ContractError> {
    let num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };
    let sha256 = Sha256::digest(
        format!("{}{}{}{}", sender, num_tokens, env.block.height, tx_index).into_bytes(),
    );
    let randomness: [u8; 32] = sha256.to_vec().try_into().unwrap();
    let r = int_in_range(randomness, 0, 50);
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
    nonpayable(&info)?;
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.extension.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }
    // If current time is after the stored start time return error
    if env.block.time >= config.extension.start_time {
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

    config.extension.start_time = start_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("sender", info.sender)
        .add_attribute("start_time", start_time.to_string()))
}

pub fn execute_update_start_trading_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<Timestamp>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    let sg721_contract_addr = SG721_ADDRESS.load(deps.storage)?;

    if info.sender != config.extension.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    // add custom rules here
    let factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory.clone(), &FactoryQueryMsg::Params {})?;
    let default_start_time_with_offset = config
        .extension
        .start_time
        .plus_seconds(factory_params.params.max_trading_offset_secs);

    if let Some(start_trading_time) = start_time {
        if env.block.time > start_trading_time {
            return Err(ContractError::InvalidStartTradingTime(
                env.block.time,
                start_trading_time,
            ));
        }
        // If new start_trading_time > old start time + offset , return error
        if start_trading_time > default_start_time_with_offset {
            return Err(ContractError::InvalidStartTradingTime(
                start_trading_time,
                default_start_time_with_offset,
            ));
        }
    }

    // execute sg721 contract
    let msg = WasmMsg::Execute {
        contract_addr: sg721_contract_addr.to_string(),
        msg: to_json_binary(&Sg721ExecuteMsg::<Empty, Empty>::UpdateStartTradingTime(
            start_time,
        ))?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("action", "update_start_trading_time")
        .add_attribute("sender", info.sender)
        .add_message(msg))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    per_address_limit: u32,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.extension.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    let factory: ParamsResponse = deps
        .querier
        .query_wasm_smart(config.factory.clone(), &FactoryQueryMsg::Params {})?;
    let factory_params = factory.params;

    if per_address_limit == 0 || per_address_limit > factory_params.max_per_address_limit {
        return Err(ContractError::InvalidPerAddressLimit {
            max: factory_params.max_per_address_limit,
            min: 1,
            got: per_address_limit,
        });
    }

    if !check_dynamic_per_address_limit(
        per_address_limit,
        config.extension.num_tokens,
        factory_params.max_per_address_limit,
    )? {
        return Err(ContractError::InvalidPerAddressLimit {
            max: display_max_mintable_tokens(
                per_address_limit,
                config.extension.num_tokens,
                factory_params.max_per_address_limit,
            )?,
            min: 1,
            got: per_address_limit,
        });
    }

    config.extension.per_address_limit = per_address_limit;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_per_address_limit")
        .add_attribute("sender", info.sender)
        .add_attribute("limit", per_address_limit.to_string()))
}

pub fn execute_burn_remaining(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    // Check only admin
    if info.sender != config.extension.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    // check mint not sold out
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    if mintable_num_tokens == 0 {
        return Err(ContractError::SoldOut {});
    }

    let keys = MINTABLE_TOKEN_POSITIONS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<Vec<_>>();
    let mut total: u32 = 0;
    for key in keys {
        total += 1;
        MINTABLE_TOKEN_POSITIONS.remove(deps.storage, key?);
    }
    // Decrement mintable num tokens
    MINTABLE_NUM_TOKENS.save(deps.storage, &(mintable_num_tokens - total))?;

    let event = Event::new("burn-remaining")
        .add_attribute("sender", info.sender)
        .add_attribute("tokens_burned", total.to_string())
        .add_attribute("minter", env.contract.address.to_string());
    Ok(Response::new().add_event(event))
}

fn mint_count(deps: Deps, address: Addr) -> Result<u32, StdError> {
    let mint_count = (MINTER_ADDRS.key(&address).may_load(deps.storage)?).unwrap_or(0);
    Ok(mint_count)
}

pub fn display_max_mintable_tokens(
    per_address_limit: u32,
    num_tokens: u32,
    max_per_address_limit: u32,
) -> Result<u32, ContractError> {
    if per_address_limit > max_per_address_limit {
        return Ok(max_per_address_limit);
    }
    if num_tokens < 100 {
        return Ok(3_u32);
    }
    let three_percent = get_three_percent_of_tokens(num_tokens)?.u128();
    Ok(three_percent as u32)
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
        QueryMsg::StartTime {} => to_json_binary(&query_start_time(deps)?),
        QueryMsg::MintableNumTokens {} => to_json_binary(&query_mintable_num_tokens(deps)?),
        QueryMsg::MintCount { address } => to_json_binary(&query_mint_count(deps, address)?),
        QueryMsg::MintTokens {} => to_json_binary(&query_mint_tokens(deps)?),
        QueryMsg::DepositedTokens { address } => {
            to_json_binary(&query_deposited_tokens(deps, address)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.extension.admin.to_string(),
        base_token_uri: config.extension.base_token_uri,
        sg721_address: sg721_address.to_string(),
        sg721_code_id: config.collection_code_id,
        num_tokens: config.extension.num_tokens,
        start_time: config.extension.start_time,
        per_address_limit: config.extension.per_address_limit,
        factory: config.factory.to_string(),
        mint_tokens: config.extension.mint_tokens,
    })
}

pub fn query_status(deps: Deps) -> StdResult<StatusResponse> {
    let status = STATUS.load(deps.storage)?;

    Ok(StatusResponse { status })
}

pub fn query_mint_tokens(deps: Deps) -> StdResult<Vec<MintToken>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.extension.mint_tokens)
}

pub fn query_deposited_tokens(deps: Deps, address: String) -> StdResult<MintTokensResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let received_tokens = RECEIVED_TOKENS
        .prefix(&addr)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (k, v) = item?;
            Ok((k, v))
        })
        .collect::<StdResult<Vec<(String, u32)>>>()?
        .iter()
        .map(|(k, v)| MintToken {
            collection: k.to_string(),
            amount: *v,
        })
        .collect::<Vec<MintToken>>();
    Ok(MintTokensResponse {
        mint_tokens: received_tokens,
    })
}

fn query_mint_count(deps: Deps, address: String) -> StdResult<MintCountResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let mint_count = (MINTER_ADDRS.key(&addr).may_load(deps.storage)?).unwrap_or(0);
    Ok(MintCountResponse {
        address: addr.to_string(),
        count: mint_count,
    })
}

fn query_start_time(deps: Deps) -> StdResult<StartTimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(StartTimeResponse {
        start_time: config.extension.start_time.to_string(),
    })
}

fn query_mintable_num_tokens(deps: Deps) -> StdResult<MintableNumTokensResponse> {
    let count = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    Ok(MintableNumTokensResponse { count })
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
            SG721_ADDRESS.save(deps.storage, &Addr::unchecked(sg721_address.clone()))?;
            Ok(Response::default()
                .add_attribute("action", "instantiate_sg721_reply")
                .add_attribute("sg721_address", sg721_address))
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let event = Event::new("migrate")
        .add_attribute("from_name", current_version.contract)
        .add_attribute("from_version", current_version.version)
        .add_attribute("to_name", CONTRACT_NAME)
        .add_attribute("to_version", CONTRACT_VERSION);
    Ok(Response::new().add_event(event))
}
