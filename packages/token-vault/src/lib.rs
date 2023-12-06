use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Metadata {
    /// cw-vesting contract
    pub balance: String,
}
