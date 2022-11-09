use cosmwasm_std::{DepsMut, MessageInfo};

use crate::msg::VerifyResponse;
use crate::signature_verify::query_verify_ethereum_text;
use crate::state::Config;

pub fn compute_valid_eth_sig(
    deps: &DepsMut,
    info: MessageInfo,
    config: &Config,
    eth_sig: String,
    eth_address: String,
) -> VerifyResponse {
    let plaintext_msg = compute_plaintext_msg(config, info);
    let eth_sig_hex = hex::decode(eth_sig).unwrap();
    query_verify_ethereum_text(deps.as_ref(), &plaintext_msg, &eth_sig_hex, &eth_address).unwrap()
}

pub fn compute_plaintext_msg(config: &Config, info: MessageInfo) -> String {
    str::replace(
        &config.claim_msg_plaintext,
        "{wallet}",
        info.sender.as_ref(),
    )
}
