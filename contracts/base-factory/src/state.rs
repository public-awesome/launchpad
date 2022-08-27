use cosmwasm_std::Empty;
use cw_storage_plus::Item;
use sg2::InitialMinterParams;

pub type Extension = Option<Empty>;

pub type BaseMinterParams = InitialMinterParams<Extension>;

pub const SUDO_PARAMS: Item<BaseMinterParams> = Item::new("sudo-params");
