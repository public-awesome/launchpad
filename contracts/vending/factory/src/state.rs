use cw_storage_plus::Item;
use vending::VendingMinterParams;

pub const SUDO_PARAMS: Item<VendingMinterParams> = Item::new("sudo-params");
