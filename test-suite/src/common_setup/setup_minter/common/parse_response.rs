use crate::common_setup::msg::MinterCollectionResponse;
use anyhow::Error;
use cosmwasm_std::Addr;
use cw_multi_test::AppResponse;

pub fn parse_factory_response(res: &AppResponse) -> (Addr, Addr) {
    let events = res.events.clone();
    let mut contract_addrs: Vec<String> = vec![];
    let vector_of_attribute_vectors = events
        .iter()
        .filter(|e| e.ty == "instantiate")
        .map(|v| v.attributes.clone())
        .collect::<Vec<_>>();
    for vector in vector_of_attribute_vectors {
        let contract_addr = vector
            .iter()
            .filter(|a| a.key == "_contract_address")
            .map(|e| e.value.clone())
            .collect::<Vec<_>>();
        contract_addrs = [contract_addrs.clone(), contract_addr].concat();
    }
    let minter_addr = Addr::unchecked(contract_addrs[0].clone());
    let collection_addr = Addr::unchecked(contract_addrs[1].clone());
    (minter_addr, collection_addr)
}

pub fn build_collection_response(
    res: Result<AppResponse, Error>,
    factory_addr: Addr,
) -> MinterCollectionResponse {
    match res.is_ok() {
        true => {
            let (minter_addr, collection_addr) = parse_factory_response(&res.unwrap());
            MinterCollectionResponse {
                minter: Some(minter_addr),
                collection: Some(collection_addr),
                factory: Some(factory_addr),
                error: None,
            }
        }
        false => MinterCollectionResponse {
            minter: None,
            collection: None,
            factory: Some(factory_addr),
            error: Some(res.unwrap_err()),
        },
    }
}
