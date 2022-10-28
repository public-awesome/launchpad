use cosmwasm_std::{Deps, StdError, StdResult};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

use crate::ethereum::{decode_address, ethereum_address_raw, get_recovery_param};
use crate::msg::VerifyResponse;

#[allow(dead_code)]
pub const VERSION: &str = "crypto-verify-v2";

#[allow(dead_code)]
pub fn query_verify_cosmos(
    deps: Deps,
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<VerifyResponse> {
    // Hashing
    let hash = Sha256::digest(message);

    // Verification
    let result = deps
        .api
        .secp256k1_verify(hash.as_ref(), signature, public_key);
    match result {
        Ok(verifies) => Ok(VerifyResponse { verifies }),
        Err(err) => Err(err.into()),
    }
}

pub fn query_verify_ethereum_text(
    deps: Deps,
    message: &str,
    signature: &[u8],
    signer_address: &str,
) -> StdResult<VerifyResponse> {
    let signer_address = decode_address(signer_address)?;

    // Hashing
    let mut hasher = Keccak256::new();
    hasher.update(format!("\x19Ethereum Signed Message:\n{}", message.len()));
    hasher.update(message);
    let hash = hasher.finalize();

    // Decompose signature
    let (v, rs) = match signature.split_last() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Signature must not be empty")),
    };
    let recovery = get_recovery_param(*v)?;

    // Verification
    let calculated_pubkey = deps.api.secp256k1_recover_pubkey(&hash, rs, recovery)?;
    let calculated_address = ethereum_address_raw(&calculated_pubkey)?;
    if signer_address != calculated_address {
        return Ok(VerifyResponse { verifies: false });
    }
    let result = deps.api.secp256k1_verify(&hash, rs, &calculated_pubkey);
    match result {
        Ok(verifies) => Ok(VerifyResponse { verifies }),
        Err(err) => Err(err.into()),
    }
}
