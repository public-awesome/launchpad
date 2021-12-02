use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Coin, ContractResult, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult,
};
use sg721::msg::CreatorResponse;
use sg721::state::Extension;
use std::{collections::HashMap, marker::PhantomData};

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    creator_querier: CreatorQuerier,
}

#[derive(Clone, Default)]
pub struct CreatorQuerier {
    creator: HashMap<String, CreatorResponse>,
}

impl CreatorQuerier {
    pub fn new(creator_info: &[(&String, &CreatorResponse)]) -> Self {
        CreatorQuerier {
            creator: creator_to_map(creator_info),
        }
    }
}

pub(crate) fn creator_to_map(
    info: &[(&String, &CreatorResponse)],
) -> HashMap<String, CreatorResponse> {
    let mut creator_map: HashMap<String, CreatorResponse> = HashMap::new();
    for (key, creator) in info.iter() {
        creator_map.insert(key.to_string(), (*creator).clone());
    }
    creator_map
}

impl Querier for WasmMockQuerier {
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

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        SystemResult::Ok(ContractResult::from(to_binary(&CreatorResponse {
            creator: Addr::unchecked("creator"),
        })))
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            creator_querier: CreatorQuerier::default(),
        }
    }

    // configure creator info
    pub fn with_creator(&mut self, creator_info: &[(&String, &CreatorResponse)]) {
        self.creator_querier = CreatorQuerier::new(creator_info);
    }
}
