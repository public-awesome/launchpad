use open_edition_factory::{
    msg::OpenEditionMinterInitMsgExtension,
    state::ParamsExtension,
    types::{NftData, NftMetadataType},
};

use crate::common_setup::msg::OpenEditionMinterInstantiateParams;

pub fn minter_params_open_edition(
    params_extension: ParamsExtension,
    per_address_limit: Option<u32>,
    init_msg: Option<OpenEditionMinterInitMsgExtension>,
) -> OpenEditionMinterInstantiateParams {
    OpenEditionMinterInstantiateParams {
        start_time: None,
        end_time: None,
        per_address_limit: per_address_limit,
        nft_data: Some(NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: Some(
                "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg"
                    .to_string(),
            ),
        }),
        init_msg,
        params_extension: Some(params_extension),
    }
}
