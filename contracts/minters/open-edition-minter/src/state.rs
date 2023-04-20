use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

use open_edition_factory::types::NftData;
use sg4::{MinterConfig, Status};

#[cw_serde]
pub struct ConfigExtension {
    pub admin: Addr,
    pub payment_address: Option<Addr>,
    pub nft_data: NftData,
    pub nb_of_nfts_minted: u32,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub end_time: Timestamp
}
pub type Config = MinterConfig<ConfigExtension>;

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
pub const MINTER_ADDRS: Map<&Addr, u32> = Map::new("ma");

/// Holds the status of the minter. Can be changed with on-chain governance proposals.
pub const STATUS: Item<Status> = Item::new("status");
