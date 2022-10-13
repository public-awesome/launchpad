use cosmwasm_std::Empty;

pub use sg721_base::ContractError;
use sg_metadata::Metadata;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-metadata-onchain";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Sg721MetadataContract<'a> = sg721_base::Sg721Contract<'a, Metadata>;
pub type InstantiateMsg = sg721::InstantiateMsg;
pub type ExecuteMsg = sg721::ExecuteMsg<Metadata, Empty>;
pub type QueryMsg = sg721_base::msg::QueryMsg;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
    use sg721_base::{msg::QueryMsg, ContractError};
    use sg_std::Response;

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Sg721MetadataContract::default().instantiate(deps, env, info, msg)?;

        Ok(res
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Sg721MetadataContract::default().execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Sg721MetadataContract::default().query(deps, env, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{
        from_slice, to_binary, ContractInfoResponse, ContractResult, Empty, OwnedDeps, Querier,
        QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
    };
    use cw721::Cw721Query;
    use cw721_base::MintMsg;
    use sg721::{CollectionInfo, ExecuteMsg, InstantiateMsg};
    use std::marker::PhantomData;

    const CREATOR: &str = "creator";

    pub fn mock_deps() -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier, Empty> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: CustomMockQuerier::new(MockQuerier::new(&[])),
            custom_query_type: PhantomData,
        }
    }

    pub struct CustomMockQuerier {
        base: MockQuerier,
    }

    impl Querier for CustomMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: bin_request.into(),
                    })
                }
            };

            self.handle_query(&request)
        }
    }

    impl CustomMockQuerier {
        pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
            match &request {
                QueryRequest::Wasm(WasmQuery::ContractInfo { contract_addr: _ }) => {
                    let response = ContractInfoResponse::new(1, CREATOR);
                    SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
                }
                _ => self.base.handle_query(request),
            }
        }

        pub fn new(base: MockQuerier<Empty>) -> Self {
            CustomMockQuerier { base }
        }
    }

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_deps();
        let contract = Sg721MetadataContract::default();

        // instantiate contract
        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
            collection_info: CollectionInfo {
                creator: CREATOR.to_string(),
                description: "this is a test".to_string(),
                image: "https://larry.engineer".to_string(),
                external_link: None,
                explicit_content: Some(false),
                start_trading_time: None,
                royalty_info: None,
            },
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        // mint token
        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            },
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        // check token contains correct metadata
        let res = contract
            .parent
            .nft_info(deps.as_ref(), token_id.into())
            .unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }
}
