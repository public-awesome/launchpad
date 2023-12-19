use cosmwasm_std::{Coin, Timestamp, Uint128};
use open_edition_factory::{
    msg::OpenEditionMinterInitMsgExtension,
    state::{OpenEditionMinterParams, ParamsExtension},
    types::{NftData, NftMetadataType},
};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use super::mock_params::mock_init_minter_extension;
use crate::common_setup::{
    msg::OpenEditionMinterInstantiateParams,
    setup_minter::common::constants::MIN_MINT_PRICE_OPEN_EDITION,
};

pub fn minter_params_open_edition(
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    nft_data: Option<NftData>,
    uri_scheme: Option<String>,
    custom_params: Option<OpenEditionMinterParams>,
) -> OpenEditionMinterInstantiateParams {
    let start_time = start_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let end_time = end_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000));
    let uri_scheme = uri_scheme.unwrap_or("ipfs".to_owned());

    OpenEditionMinterInstantiateParams {
        start_time: Some(start_time),
        end_time: Some(end_time),
        per_address_limit: Some(init_msg.per_address_limit),
        nft_data: Some(nft_data.unwrap_or(NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: Some(format!(
                "{}://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg",
                uri_scheme
            )),
        })),
        init_msg: Some(init_msg),
        params_extension: Some(params_extension),
        custom_params,
    }
}

pub fn default_nft_data() -> NftData {
    NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        extension: None,
        token_uri: Some(
            "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg".to_string(),
        ),
    }
}

pub fn nft_data_with_uri_scheme(uri_scheme: String) -> NftData {
    NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        extension: None,
        token_uri: Some(format!(
            "{}://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg",
            uri_scheme
        )),
    }
}

pub fn init_msg(
    nft_data: NftData,
    per_address_limit_minter: Option<u32>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    mint_price: Option<Coin>,
) -> OpenEditionMinterInitMsgExtension {
    let start_time = start_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let end_time = end_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000));
    let mint_price = mint_price.unwrap_or(Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
    });
    mock_init_minter_extension(
        Some(start_time),
        Some(end_time),
        per_address_limit_minter,
        Some(mint_price),
        nft_data,
        None,
    )
}
