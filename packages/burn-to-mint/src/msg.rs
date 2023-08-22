use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct TokenUriMsg {
    pub token_uri: String,
}
