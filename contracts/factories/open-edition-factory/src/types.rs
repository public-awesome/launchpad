use cosmwasm_schema::cw_serde;
use cosmwasm_std::ensure;

use sg_metadata::Metadata;

use crate::ContractError;

// TODO: These types should be put in a more general module.

#[cw_serde]
pub enum NftMetadataType {
    OnChainMetadata,
    OffChainMetadata,
}

#[cw_serde]
pub struct NftData {
    pub nft_data_type: NftMetadataType,
    pub extension: Option<Metadata>,
    pub token_uri: Option<String>,
}

impl NftData {
    pub fn validate(nft_data: NftData) -> Result<Self, ContractError> {
        ensure!(
            nft_data.valid_nft_data(),
            ContractError::InvalidNftDataProvided {}
        );

        // Validation of the metadata and token_uri is validated at the nft contract level

        Ok(NftData {
            nft_data_type: nft_data.nft_data_type,
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
