use cosmwasm_std::{DepsMut, MessageInfo, StdError, StdResult};

use crate::ethereum::verify_ethereum_text;
use crate::msg::VerifyResponse;
use crate::state::Config;

pub fn compute_valid_eth_sig(
    deps: &DepsMut,
    info: MessageInfo,
    config: &Config,
    eth_sig: String,
    eth_address: String,
) -> StdResult<VerifyResponse> {
    let plaintext_msg = compute_plaintext_msg(config, info);
    match hex::decode(eth_sig.clone()) {
        Ok(eth_sig_hex) => {
            verify_ethereum_text(deps.as_ref(), &plaintext_msg, &eth_sig_hex, &eth_address)
        }
        Err(_) => Err(StdError::InvalidHex {
            msg: format!("Could not decode {}", eth_sig),
        }),
    }
}

pub fn compute_plaintext_msg(config: &Config, info: MessageInfo) -> String {
    str::replace(
        &config.claim_msg_plaintext,
        "{wallet}",
        info.sender.as_ref(),
    )
}
