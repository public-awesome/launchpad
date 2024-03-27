use sg_multi_test::StargazeApp;
use whitelist_immutable_flex::msg::Member;

pub struct InstantiateParams<'a> {
    pub members: Vec<Member>,
    pub funds_amount: u128,
    pub expected_airdrop_contract_id: u64,
    pub minter_address: String,
    pub admin_account: String,
    pub app: &'a mut StargazeApp,
    pub name_discount_wl_address: String,
    pub name_collection_address: String,
    pub airdrop_count_limit: u32,
    pub claim_msg_plaintext: String,
    pub airdrop_amount: u128,
}
