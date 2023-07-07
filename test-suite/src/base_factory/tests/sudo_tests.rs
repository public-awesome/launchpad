use crate::common_setup::setup_minter::base_minter::setup::sudo_update_params;
use crate::common_setup::templates::vending_minter_with_sudo_update_params_template;

#[test]
fn happy_path_with_params_update() {
    let vt = vending_minter_with_sudo_update_params_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    sudo_update_params(&mut router, &vt.collection_response_vec, vt.code_ids);
}
