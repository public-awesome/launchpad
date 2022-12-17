use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, QuerierWrapper};
use cw4::Cw4Contract;

use cw_storage_plus::Item;

use crate::ContractError;

#[cw_serde]
pub enum Executor {
    /// Any member of the voting group, even with 0 points
    Member,
    /// Only the given address
    Only(Addr),
}

#[cw_serde]
pub struct Config {
    // Total weight and members are queried from this contract
    pub group_addr: Cw4Contract,
    // Who is able to call distribute
    // None means that anyone can call distribute
    pub executor: Option<Executor>,
}

// unique items
pub const CONFIG: Item<Config> = Item::new("config");

impl Config {
    // Executor can be set in 3 ways:
    // - Member: any member of the voting group is authorized
    // - Only: only passed address is authorized
    // - None: Everyone are authorized
    pub fn authorize(&self, querier: &QuerierWrapper, sender: &Addr) -> Result<(), ContractError> {
        if let Some(executor) = &self.executor {
            match executor {
                Executor::Member => {
                    self.group_addr
                        .is_member(querier, sender, None)?
                        .ok_or(ContractError::Unauthorized {})?;
                }
                Executor::Only(addr) => {
                    if addr != sender {
                        return Err(ContractError::Unauthorized {});
                    }
                }
            }
        }
        Ok(())
    }
}
