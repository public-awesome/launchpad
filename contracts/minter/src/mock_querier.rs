use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Coin, ContractResult, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult,
};
use sg721::state::CreatorInfo;
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
    creator_info_querier: CreatorInfoQuerier,
}

#[derive(Clone, Default)]
pub struct CreatorInfoQuerier {
    creator_info: HashMap<String, CreatorInfo>,
}

impl CreatorInfoQuerier {
    pub fn new(creator_info: &[(&String, &CreatorInfo)]) -> Self {
        CreatorInfoQuerier {
            creator_info: creator_info_to_map(creator_info),
        }
    }
}

pub(crate) fn creator_info_to_map(
    info: &[(&String, &CreatorInfo)],
) -> HashMap<String, CreatorInfo> {
    let mut creator_info_map: HashMap<String, CreatorInfo> = HashMap::new();
    for (key, creator) in info.iter() {
        creator_info_map.insert(key.to_string(), (*creator).clone());
    }
    creator_info_map
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
        SystemResult::Ok(ContractResult::from(to_binary(&CreatorInfo {
            creator: Addr::unchecked("creator"),
            creator_share: 50u64,
        })))
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            creator_info_querier: CreatorInfoQuerier::default(),
        }
    }

    // configure creator info
    pub fn with_creator_info(&mut self, creator_info: &[(&String, &CreatorInfo)]) {
        self.creator_info_querier = CreatorInfoQuerier::new(creator_info);
    }
}
