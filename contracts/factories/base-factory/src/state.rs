use cosmwasm_std::Empty;
use cw_storage_plus::Item;
use sg2::MinterParams;

pub type Extension = Option<Empty>;

pub type BaseMinterParams = MinterParams<Extension>;

pub const SUDO_PARAMS: Item<BaseMinterParams> = Item::new("sudo-params");

// migration version constants
pub const EARLIEST_VERSION: &str = "2.1.0";
pub const TO_VERSION: &str = "3.0.0";
