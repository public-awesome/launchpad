use cosmwasm_std::{DepsMut, StdResult};
use cw_storage_plus::Item;
use sg3::Status;
use sg_std::Response;

pub const STATUS: Item<Status> = Item::new("status");

/// Only governance can update contract params
pub fn update_status(deps: DepsMut, verified: bool, blocked: bool) -> StdResult<Response> {
    // let _: StdResult<Minter> = MINTERS.update(deps.storage, &minter_addr, |m| match m {
    //     None => Ok(Minter { verified, blocked }),
    //     Some(mut m) => {
    //         m.verified = verified;
    //         m.blocked = blocked;
    //         Ok(m)
    //     }
    // });

    let status = STATUS.may_load(deps.storage)?;
    status.verified = verified;

    Ok(Response::new().add_attribute("action", "sudo_update_minter_status"))
}
