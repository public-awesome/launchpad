use cosmwasm_schema::cw_serde;
use cosmwasm_std::ensure;

use sg_metadata::Metadata;

use crate::ContractError;

// TODO: These types should be put in a more general module.

#[cw_serde]
pub enum NftMetadataType {
    OnChainMetadata,
    OffChainMetadata
}

#[cw_serde]
pub struct NftData {
    pub nft_data_type: NftMetadataType,
    pub token_id_prefix: String,
    pub extension: Option<Metadata>,
    pub token_uri: Option<String>
}

impl NftData {
    pub fn new_validated(
        nft_data: NftData,
        token_id_prefix_length: u32
    ) -> Result<Self, ContractError> {

        ensure!(nft_data.valid_nft_data(), ContractError::InvalidNftDataProvided {});

        // Validation of the metadata and token_uri is validated at the nft contract level

        // The token id prefix is just the name of the token id which will be concatenated with the NFT counter
        // f.e.: token_id_prefix = "Stargaze NFT #" and there is 10000 NFTs to be minted, we would have
        // "Stargaze NFT #00001", "Stargaze NFT #00002", ... "Stargaze NFT #09999", "Stargaze NFT #10000",
        if nft_data.token_id_prefix.len() > token_id_prefix_length as usize {
            return Err(ContractError::TokenIdPrefixIsTooLong {})
        }
        Ok(NftData {
            nft_data_type: nft_data.nft_data_type,
            token_id_prefix: nft_data.token_id_prefix,
            extension: nft_data.extension,
            token_uri: nft_data.token_uri,
        })
    }

    pub fn valid_nft_data(&self) -> bool {
        if self.token_uri.is_some() && self.extension.is_some() {
            return false;
        }
        if self.token_uri.is_some() && self.nft_data_type == NftMetadataType::OffChainMetadata {
            true
        } else {
            self.extension.is_some() && self.nft_data_type == NftMetadataType::OnChainMetadata
        }
    }

}