use cosmwasm_std::Addr;

use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::setup_minter::base_minter::setup::sudo_update_params;
use crate::common_setup::templates::vending_minter_with_sudo_update_params_template;

#[test]
fn happy_path_with_params_update() {
    let vt = vending_minter_with_sudo_update_params_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    sudo_update_params(&mut router, &vt.collection_response_vec, vt.code_ids);
}

#[test]
fn params_update_invalid_nft_collection() {
    let vt = vending_minter_with_sudo_update_params_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let mut response_collection_vec: Vec<MinterCollectionResponse> = vec![];
    for entry in vt.collection_response_vec {
        let new_entry = MinterCollectionResponse {
            minter: entry.minter,
            collection: Some(Addr::unchecked("fake_address")),
            factory: entry.factory,
            error: entry.error,
        };
        response_collection_vec.push(new_entry);
    }
    let sudo_responses = sudo_update_params(&mut router, &response_collection_vec, vt.code_ids);
    let sudo_response_1 = sudo_responses.first();
    let err = sudo_response_1.unwrap().as_ref().unwrap_err().to_string();
    assert_eq!(err, "InvalidCollectionAddress".to_string());
}
