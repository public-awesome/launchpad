use crate::sg_eth_airdrop::constants::claim_constants::CONFIG_PLAINTEXT;
use async_std::task;
use ethers_core::{k256::ecdsa::SigningKey, rand::thread_rng, types::H160};
use ethers_signers::{LocalWallet, Signer, Wallet, WalletError};

pub async fn get_signature(
    wallet: Wallet<SigningKey>,
    plaintext_msg: &str,
) -> Result<ethers_core::types::Signature, WalletError> {
    wallet.sign_message(plaintext_msg).await
}

pub fn get_wallet_and_sig(
    claim_plaintext: String,
) -> (
    Wallet<ethers_core::k256::ecdsa::SigningKey>,
    std::string::String,
    H160,
    std::string::String,
) {
    let wallet = LocalWallet::new(&mut thread_rng());
    let eth_sig_str = task::block_on(get_signature(wallet.clone(), &claim_plaintext))
        .unwrap()
        .to_string();
    let eth_address = wallet.address();
    let eth_addr_str = format!("{eth_address:?}");
    (wallet, eth_sig_str, eth_address, eth_addr_str)
}

pub fn get_msg_plaintext(wallet_address: String) -> String {
    str::replace(CONFIG_PLAINTEXT, "{wallet}", &wallet_address)
}
