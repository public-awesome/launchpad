use cw4::Cw4Contract;

use cw_controllers::Admin;
use cw_storage_plus::Item;

pub const GROUP: Item<Cw4Contract> = Item::new("group");

pub const ADMIN: Admin = Admin::new("admin");
