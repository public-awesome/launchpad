use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub claim_msg_plaintext: String,
    pub airdrop_amount: u128,
    pub addresses: Vec<String>,
    pub whitelist_code_id: u64,
    pub minter_address: Addr,
    pub per_address_limit: u32,
}

#[cw_serde]
pub struct AirdropClaimResponse {
    result: bool,
    amount: u32,
    minter_page: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimAirdrop {
        eth_address: String,
        eth_sig: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    AirdropEligible { eth_address: String },
    #[returns(Addr)]
    GetMinter {},
}
