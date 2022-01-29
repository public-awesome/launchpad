#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty,
    Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_utils::parse_reply_instantiate_data;
use sg721::msg::{InstantiateMsg as Sg721InstantiateMsg, QueryMsg as Sg721QueryMsg};

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, SG721_ADDRESS, TOKEN_ID_INDEX, TOKEN_URIS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sale";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_SG721_REPLY_ID: u64 = 1;

const MAX_TOKEN_URIS_LENGTH: u32 = 15000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Check token uris list length doesn't exceed max
    if msg.token_uris.len() > MAX_TOKEN_URIS_LENGTH as usize {
        return Err(ContractError::MaxTokenURIsLengthExceeded {});
    }

    // Check length of token uris is not greater than max tokens
    if msg.token_uris.len() != msg.num_tokens as usize {
        return Err(ContractError::TokenURIsListInvalidNumber {});
    }

    // Map through list of token URIs
    for (index, token_uri) in msg.token_uris.into_iter().enumerate() {
        TOKEN_URIS.save(deps.storage, index as u64, &token_uri)?;
    }

    let config = Config {
        admin: info.sender.clone(),
        num_tokens: msg.num_tokens,
        sg721_code_id: msg.sg721_code_id,
        unit_price: msg.unit_price,
    };
    CONFIG.save(deps.storage, &config)?;

    // Set Token ID index
    TOKEN_ID_INDEX.save(deps.storage, &0)?;

    let sub_msgs: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.sg721_code_id,
            msg: to_binary(&Sg721InstantiateMsg {
                name: msg.sg721_instantiate_msg.name,
                symbol: msg.sg721_instantiate_msg.symbol,
                minter: env.contract.address.to_string(),
                collection_info: msg.sg721_instantiate_msg.collection_info,
            })?,
            funds: vec![],
            admin: None,
            label: String::from("Instantiate fixed price NFT contract"),
        }
        .into(),
        id: INSTANTIATE_SG721_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
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
        ExecuteMsg::Mint {} => execute_mint(deps, env, info),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let mut token_id_index = TOKEN_ID_INDEX.load(deps.storage)?;
    let token_uri = TOKEN_URIS.load(deps.storage, token_id_index)?;

    // Check funds sent is correct amount
    if !has_coins(&info.funds, &config.unit_price) {
        return Err(ContractError::NotEnoughFunds {});
    }

    // Check if over max tokens
    if token_id_index >= config.num_tokens {
        return Err(ContractError::SoldOut {});
    }

    let mut msgs: Vec<CosmosMsg> = vec![];

    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Empty> {
        token_id: token_id_index.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(token_uri),
        extension: Empty {},
    });

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sg721_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    msgs.append(&mut vec![msg]);

    // Increase token ID index by one
    token_id_index += 1;
    TOKEN_ID_INDEX.save(deps.storage, &token_id_index)?;

    // Check if token supports Royalties
    let royalty: Result<sg721::msg::RoyaltyResponse, StdError> = deps
        .querier
        .query_wasm_smart(sg721_address, &Sg721QueryMsg::Royalties {});

    // Add payout messages
    match royalty {
        Ok(royalty) => {
            // If token supports royalities, payout shares
            if let Some(royalty) = royalty.royalty {
                // Can't assume index 0 of index.funds is the correct coin
                let funds = info.funds.iter().find(|x| *x == &config.unit_price);
                if let Some(funds) = funds {
                    // Calculate royalty share and create Bank msg
                    let royalty_share_msg = BankMsg::Send {
                        to_address: royalty.payment_address.to_string(),
                        amount: vec![Coin {
                            amount: funds.amount * royalty.share,
                            denom: funds.denom.clone(),
                        }],
                    };
                    msgs.append(&mut vec![royalty_share_msg.into()]);

                    // Calculate seller share and create Bank msg
                    let seller_share_msg = BankMsg::Send {
                        to_address: config.admin.to_string(),
                        amount: vec![Coin {
                            amount: funds.amount * (Decimal::one() - royalty.share),
                            denom: funds.denom.clone(),
                        }],
                    };
                    msgs.append(&mut vec![seller_share_msg.into()]);
                }
            }
        }
        Err(_) => {
            // If token doesn't support royalties, pay seller in full
            let seller_share_msg = BankMsg::Send {
                to_address: config.admin.to_string(),
                amount: info.funds,
            };
            msgs.append(&mut vec![seller_share_msg.into()]);
        }
    }

    Ok(Response::default().add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let unused_token_id = TOKEN_ID_INDEX.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        sg721_address,
        sg721_code_id: config.sg721_code_id,
        num_tokens: config.num_tokens,
        unit_price: config.unit_price,
        unused_token_id,
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
            Ok(Response::default())
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Decimal};
    use cw721::{Cw721QueryMsg, OwnerOfResponse};
    use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use sg721::state::{CollectionInfo, RoyaltyInfo};

    const DENOM: &str = "ustars";
    const INITIAL_BALANCE: u128 = 2000;
    const PRICE: u128 = 10;

    fn mock_app() -> App {
        App::default()
    }

    pub fn contract_sale() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn contract_sg721() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            sg721::contract::execute,
            sg721::contract::instantiate,
            sg721::contract::query,
        );
        Box::new(contract)
    }

    // Upload contract code and instantiate sale contract
    fn setup_sale_contract(
        router: &mut App,
        creator: &Addr,
    ) -> Result<(Addr, ConfigResponse), ContractError> {
        // Upload contract code
        let sg721_code_id = router.store_code(contract_sg721());
        let sale_code_id = router.store_code(contract_sale());

        // Instantiate sale contract
        let msg = InstantiateMsg {
            unit_price: coin(PRICE, DENOM),
            num_tokens: 1,
            token_uris: vec![String::from("https://stargaze.zone/logo.png")],
            sg721_code_id,
            sg721_instantiate_msg: Sg721InstantiateMsg {
                name: String::from("TEST"),
                symbol: String::from("TEST"),
                minter: creator.to_string(),
                collection_info: CollectionInfo {
                    contract_uri: String::from("test"),
                    creator: creator.clone(),
                    royalties: Some(RoyaltyInfo {
                        payment_address: creator.clone(),
                        share: Decimal::percent(10),
                    }),
                },
            },
        };
        let sale_addr = router
            .instantiate_contract(sale_code_id, creator.clone(), &msg, &[], "Sale", None)
            .unwrap();

        let config: ConfigResponse = router
            .wrap()
            .query_wasm_smart(sale_addr.clone(), &QueryMsg::GetConfig {})
            .unwrap();

        Ok((sale_addr, config))
    }

    // Add a creator account with initial balances
    fn setup_accounts(router: &mut App) -> Result<(Addr, Addr), ContractError> {
        let buyer: Addr = Addr::unchecked("buyer");
        let creator: Addr = Addr::unchecked("creator");
        let funds: Vec<Coin> = coins(INITIAL_BALANCE, DENOM);
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: creator.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: buyer.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        // Check native balances
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(creator_native_balances, funds);

        // Check native balances
        let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
        assert_eq!(buyer_native_balances, funds);

        Ok((creator, buyer))
    }

    #[test]
    fn initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // Num tokens does not match token_uris length and should error
        let info = mock_info("creator", &coins(INITIAL_BALANCE, DENOM));
        let msg = InstantiateMsg {
            unit_price: coin(PRICE, DENOM),
            num_tokens: 100,
            token_uris: vec![String::from("https://stargaze.zone/logo.png")],
            sg721_code_id: 1,
            sg721_instantiate_msg: Sg721InstantiateMsg {
                name: String::from("TEST"),
                symbol: String::from("TEST"),
                minter: info.sender.to_string(),
                collection_info: CollectionInfo {
                    contract_uri: String::from("test"),
                    creator: info.sender.clone(),
                    royalties: Some(RoyaltyInfo {
                        payment_address: info.sender.clone(),
                        share: Decimal::percent(10),
                    }),
                },
            },
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err())
    }

    #[test]
    fn happy_path() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let (sale_addr, config) = setup_sale_contract(&mut router, &creator).unwrap();

        // Succeeds if funds are sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // Balances are correct
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE + PRICE, DENOM)
        );
        let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
        assert_eq!(buyer_native_balances, coins(INITIAL_BALANCE - PRICE, DENOM));

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: String::from("0"),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(config.sg721_address, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, buyer.to_string());

        // Errors if sold out
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(buyer, sale_addr, &mint_msg, &coins(PRICE, DENOM));
        assert!(res.is_err());
    }

    #[test]
    fn unhappy_path() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let (sale_addr, _config) = setup_sale_contract(&mut router, &creator).unwrap();

        // Fails if too little funds are sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(1, DENOM),
        );
        assert!(res.is_err());

        // Fails if too many funds are sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(11111, DENOM),
        );
        assert!(res.is_err());

        // Fails wrong denom is sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(buyer, sale_addr, &mint_msg, &coins(PRICE, "uatom"));
        assert!(res.is_err());
    }
}
