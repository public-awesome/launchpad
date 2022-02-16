#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, has_coins, to_binary, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty,
    Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ContractError;
use cw721::ContractInfoResponse;
use cw721_base::ContractError as BaseError;

use crate::msg::{
    ContractUriResponse, CreatorResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyResponse,
};
use crate::state::CONFIG;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: these should be a governance parameters in the future
const FEE_DENOM: &str = "ustars";
const CREATION_FEE: u128 = 1_000_000_000;
const CREATION_FEE_BURN_PERCENT: u64 = 50;

pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, Empty>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MsgFundCommunityPool {
    pub amount: Vec<Coin>,
    pub depositor: String,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let payment = must_pay(&info, FEE_DENOM)?;
    if payment.u128() != CREATION_FEE {
        return Err(ContractError::InvalidCreationFee {});
    }

    // calculate the fee to burn
    let burn_percent = Decimal::percent(CREATION_FEE_BURN_PERCENT);
    let creation_fee = Uint128::from(CREATION_FEE);
    let burn_fee = creation_fee * burn_percent;
    let burn_coin = coin(burn_fee.u128(), FEE_DENOM);
    // send fee to contract to be burned
    let send_fee_msg = BankMsg::Send {
        to_address: env.contract.address.to_string(),
        amount: vec![burn_coin.clone()],
    };
    // burn half the fee
    let fee_burn_msg = BankMsg::Burn {
        amount: vec![burn_coin],
    };

    // TODO: send the rest to the community pool
    // https://github.com/public-awesome/contracts/issues/99
    let fund_community_pool_msg = CosmosMsg::Custom(MsgFundCommunityPool {
        amount: vec![coin(creation_fee.u128() - burn_fee.u128(), FEE_DENOM)],
        depositor: msg.minter.to_string(),
    });

    let info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    Sg721Contract::default()
        .contract_info
        .save(deps.storage, &info)?;
    let minter = deps.api.addr_validate(&msg.minter)?;
    Sg721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    if let Some(ref config) = msg.config {
        if let Some(ref royalty) = config.royalties {
            royalty.is_valid()?;
        }
        CONFIG.save(deps.storage, config)?;
    }

    // TODO: add community pool fund msg
    // https://github.com/public-awesome/contracts/issues/99
    Ok(Response::default()
        .add_message(send_fee_msg)
        .add_message(fee_burn_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, BaseError> {
    Sg721Contract::default().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractUri {} => to_binary(&query_contract_uri(deps)?),
        QueryMsg::Creator {} => to_binary(&query_creator(deps)?),
        QueryMsg::Royalties {} => to_binary(&query_royalties(deps)?),
        _ => Sg721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_contract_uri(deps: Deps) -> StdResult<ContractUriResponse> {
    let contract_uri = CONFIG.load(deps.storage)?.contract_uri;
    Ok(ContractUriResponse { contract_uri })
}

fn query_creator(deps: Deps) -> StdResult<CreatorResponse> {
    let creator = CONFIG.load(deps.storage)?.creator;
    Ok(CreatorResponse { creator })
}

fn query_royalties(deps: Deps) -> StdResult<RoyaltyResponse> {
    let royalty = CONFIG.load(deps.storage)?.royalties;
    Ok(RoyaltyResponse { royalty })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::Config;
    use crate::state::RoyaltyInfo;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Decimal};

    #[test]
    fn proper_initialization_no_royalties() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            config: Some(Config {
                contract_uri: Some(String::from("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json")),
                creator: Some(Addr::unchecked(creator)),
                royalties: None,
            }),
        };
        let info = mock_info("creator", &coins(CREATION_FEE, "ustars"));

        // make sure instantiate has the burn messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // it worked, let's query the contract_uri
        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractUri {}).unwrap();
        let value: ContractUriResponse = from_binary(&res).unwrap();
        assert_eq!(Some("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json".to_string()), value.contract_uri);

        // it worked, let's query the creator
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Creator {}).unwrap();

        let value: CreatorResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.creator.unwrap().to_string());

        // let's query the royalties
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Royalties {}).unwrap();
        let value: RoyaltyResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.royalty);
    }

    #[test]
    fn proper_initialization_with_royalties() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            config: Some(Config {
                contract_uri: Some(String::from("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json")),
                creator: Some(Addr::unchecked(creator.clone())),
                royalties: Some(RoyaltyInfo {
                    payment_address: Addr::unchecked(creator.clone()),
                    share: Decimal::percent(10),
                }),
            }),
        };
        let info = mock_info("creator", &coins(CREATION_FEE, "ustars"));

        // make sure instantiate has the burn messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // it worked, let's query the contract_uri
        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractUri {}).unwrap();
        let value: ContractUriResponse = from_binary(&res).unwrap();
        assert_eq!(Some("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json".to_string()), value.contract_uri);

        // it worked, let's query the creator
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Creator {}).unwrap();

        let value: CreatorResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.creator.unwrap().to_string());

        // let's query the royalties
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Royalties {}).unwrap();
        let value: RoyaltyResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(RoyaltyInfo {
                payment_address: Addr::unchecked(creator),
                share: Decimal::percent(10),
            }),
            value.royalty
        );
    }

    #[test]
    fn creation_fee() {
        let burn_percent = Decimal::percent(50);
        let creation_fee = Uint128::from(1_000_000_000u128);
        let fee = creation_fee * burn_percent;
        let amount = coin(fee.u128(), "ustars");
        assert_eq!(500_000_000u128, amount.amount.u128());
    }
    // TODO: properly test fee burn using cw-multi-test
    // set an initial supply in bank, and check that the fee is burned
    // https://github.com/public-awesome/contracts/issues/114
    #[test]
    fn fee_burn() {}
}
