use cw4::Cw4Contract;

use cw_storage_plus::Item;

/// The group that holds splits members
/// Total weight and voters are queried from this contract
pub const GROUP: Item<Cw4Contract> = Item::new("group");
