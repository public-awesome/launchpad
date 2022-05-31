use cw_controllers::Admin;

use cw_storage_plus::Item;
use sg_marketplace::MarketplaceContract;

pub const ADMIN: Admin = Admin::new("admin");

pub const MARKETPLACE: Item<MarketplaceContract> = Item::new("marketplace");
