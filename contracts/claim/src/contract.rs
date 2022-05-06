#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;
use cw_utils::maybe_addr;
use minter::msg::{MintCountResponse, QueryMsg as MinterQueryMsg};
use sg_marketplace::msg::SaleHookMsg;
use sg_marketplace::MarketplaceContract;
use sg_std::{create_claim_for_msg, ClaimAction, StargazeMsgWrapper};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MarketplaceResponse, QueryMsg};
use crate::state::{ADMIN, MARKETPLACE};
pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-claim";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    ADMIN.set(deps.branch(), maybe_addr(api, msg.admin)?)?;

    if let Some(marketplace_addr) = msg.marketplace_addr {
        let marketplace =
            MarketplaceContract(deps.api.addr_validate(&marketplace_addr).map_err(|_| {
                ContractError::InvalidMarketplace {
                    addr: marketplace_addr.clone(),
                }
            })?);
        MARKETPLACE.save(deps.storage, &marketplace)?;
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::ClaimMintNFT { minter_address } => {
            execute_claim_mint_nft(deps, info.sender, minter_address)
        }
        ExecuteMsg::SaleHook(SaleHookMsg {
            collection,
            token_id,
            price,
            seller,
            buyer,
        }) => execute_claim_buy_nft(deps, info, collection, token_id, price, seller, buyer),
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::UpdateMarketplace { marketplace_addr } => {
            execute_update_marketplace(deps, info, marketplace_addr)
        }
    }
}

/// Only the admin can update the marketplace address
pub fn execute_update_marketplace(
    deps: DepsMut,
    info: MessageInfo,
    marketplace_addr: Option<String>,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    match marketplace_addr {
        Some(marketplace_addr) => {
            let marketplace =
                MarketplaceContract(deps.api.addr_validate(&marketplace_addr).map_err(|_| {
                    ContractError::InvalidMarketplace {
                        addr: marketplace_addr.clone(),
                    }
                })?);
            MARKETPLACE.save(deps.storage, &marketplace)?;
        }
        None => {
            MARKETPLACE.remove(deps.storage);
        }
    }

    Ok(Response::new()
        .add_attribute("action", "update_marketplace")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn execute_claim_mint_nft(
    deps: DepsMut,
    sender: Addr,
    minter: String,
) -> Result<Response, ContractError> {
    let minter_addr = deps.api.addr_validate(&minter)?;
    let count_response: MintCountResponse = deps.querier.query_wasm_smart(
        minter_addr,
        &MinterQueryMsg::MintCount {
            address: sender.to_string(),
        },
    )?;
    if count_response.count == 0 {
        return Err(ContractError::NoMinting {});
    }

    let msg = create_claim_for_msg(sender.to_string(), ClaimAction::MintNFT);
    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "claim_mint_nft")
        .add_attribute("sender", sender.to_string())
        .add_attribute("minter", minter))
}

pub fn execute_claim_buy_nft(
    deps: DepsMut,
    info: MessageInfo,
    collection: String,
    token_id: u32,
    price: Coin,
    seller: String,
    buyer: String,
) -> Result<Response, ContractError> {
    let marketplace = MARKETPLACE.load(deps.storage)?;

    if info.sender != marketplace.addr() {
        return Err(ContractError::Unauthorized {});
    }

    let buyer = deps.api.addr_validate(&buyer)?;
    let msg = create_claim_for_msg(buyer.to_string(), ClaimAction::BidNFT);

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "claim_buy_nft")
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("price", price.to_string())
        .add_attribute("seller", seller)
        .add_attribute("buyer", buyer);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Marketplace {} => to_binary(&query_marketplace(deps)?),
    }
}

fn query_marketplace(deps: Deps) -> StdResult<MarketplaceResponse> {
    let marketplace = MARKETPLACE.may_load(deps.storage)?;

    Ok(MarketplaceResponse {
        marketplace: marketplace.map(|marketplace| marketplace.addr()),
    })
}
