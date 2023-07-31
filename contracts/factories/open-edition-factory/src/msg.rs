use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Deps, Env, Timestamp};

use sg2::msg::{CreateMinterMsg, Sg2ExecuteMsg, UpdateMinterParamsMsg};

use crate::state::OpenEditionMinterParams;
use crate::types::NftData;
use crate::ContractError;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: OpenEditionMinterParams,
}

#[cw_serde]
pub struct OpenEditionMinterInitMsgExtension {
    pub nft_data: NftData,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    // If not the admin/init
    pub payment_address: Option<String>,
}

impl OpenEditionMinterInitMsgExtension {
    pub fn validate(
        mut init_msg: OpenEditionMinterInitMsgExtension,
        env: Env,
        _deps: Deps,
        params: &OpenEditionMinterParams,
    ) -> Result<Self, ContractError> {
        // Validation of the Minter Params -> need to be in-line with the factory
        init_msg.nft_data = NftData::validate(init_msg.nft_data)?;

        let max = params.extension.max_per_address_limit;
        let min = 1;
        let per_address_limit = init_msg.per_address_limit;
        if init_msg.per_address_limit < min || init_msg.per_address_limit > max {
            return Err(ContractError::InvalidPerAddressLimit {
                max,
                min,
                got: per_address_limit,
            });
        }

        if init_msg.start_time <= env.block.time {
            return Err(ContractError::InvalidStartTime(
                init_msg.start_time,
                env.block.time,
            ));
        }

        if init_msg.end_time <= init_msg.start_time {
            return Err(ContractError::InvalidEndTime(
                init_msg.start_time,
                init_msg.end_time,
            ));
        }

        if init_msg.mint_price.amount < params.min_mint_price.amount {
            return Err(ContractError::InvalidMintPrice {});
        }

        Ok(OpenEditionMinterInitMsgExtension {
            nft_data: init_msg.nft_data,
            start_time: init_msg.start_time,
            end_time: init_msg.end_time,
            mint_price: init_msg.mint_price,
            per_address_limit,
            payment_address: init_msg.payment_address,
        })
    }
}

pub type OpenEditionMinterCreateMsg = CreateMinterMsg<OpenEditionMinterInitMsgExtension>;

pub type ExecuteMsg = Sg2ExecuteMsg<OpenEditionMinterInitMsgExtension>;

#[cw_serde]
pub enum SudoMsg {
    UpdateParams(Box<OpenEditionUpdateParamsMsg>),
}

/// Message for params so they can be updated individually by governance
#[cw_serde]
pub struct OpenEditionUpdateParamsExtension {
    pub max_per_address_limit: Option<u32>,
    pub min_mint_price: Option<Coin>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub airdrop_mint_price: Option<Coin>,
    pub dev_fee_address: Option<String>,
}
pub type OpenEditionUpdateParamsMsg = UpdateMinterParamsMsg<OpenEditionUpdateParamsExtension>;

#[cw_serde]
pub struct ParamsResponse {
    pub params: OpenEditionMinterParams,
}
