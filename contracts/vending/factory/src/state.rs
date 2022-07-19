use cw_storage_plus::Item;
use sg2_vending::VendingMinterParams;

pub const SUDO_PARAMS: Item<VendingMinterParams> = Item::new("sudo-params");
