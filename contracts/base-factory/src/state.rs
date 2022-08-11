use cosmwasm_std::Empty;
use cw_storage_plus::Item;
use sg2::MinterParams;

pub type BaseMinterParams = MinterParams<Empty>;

pub const SUDO_PARAMS: Item<BaseMinterParams> = Item::new("sudo-params");
