use cosmwasm_schema::cw_serde;

/// StargazeRoute is enum type to represent stargaze query route path
#[cw_serde]
pub enum StargazeRoute {
    Alloc,
    Claim,
    Distribution,
}
