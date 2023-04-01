use crate::common_setup::templates::vending_minter_with_ibc_asset;

#[test]
fn update_mint_price() {
    let num_tokens = 7000;
    let per_address_limit = 10;
    let vt = vending_minter_with_ibc_asset(num_tokens, per_address_limit);
    let (mut router, _) = (vt.router, vt.accts.creator);
    let err = vt.collection_response_vec[0].error.as_ref();
    println!("err: {:?}", err);
}
