use cosmwasm_std::Timestamp;
use cw721_base::msg::QueryMsg as Cw721QueryMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::RoyaltyInfoResponse;

// impl From<ContractError> for Cw721ContractError {
//     fn from(err: ContractError) -> Cw721ContractError {
//         match err {
//             ContractError::Unauthorized {} => Cw721ContractError::Unauthorized {},
//             ContractError::Claimed {} => Cw721ContractError::Claimed {},
//             ContractError::Expired {} => Cw721ContractError::Expired {},
//             _ => unreachable!("cannot convert {:?} to Cw721ContractError", err),
//         }
//     }
// }

// the trait `From<sg721::ExecuteMsg<T>>` is not implemented for `cw721_base::ExecuteMsg<T>`

// impl<T> From<sg721::ExecuteMsg<T>> for cw721_base::ExecuteMsg<T> {
//     fn from(msg: sg721::ExecuteMsg<T>) -> Self {
//         match msg {
//             sg721::ExecuteMsg::TransferNft {
//                 recipient,
//                 token_id,
//             } => cw721_base::ExecuteMsg::TransferNft {
//                 recipient,
//                 token_id,
//             },
//             sg721::ExecuteMsg::SendNft {
//                 contract,
//                 token_id,
//                 msg,
//             } => todo!(),
//             sg721::ExecuteMsg::Approve {
//                 spender,
//                 token_id,
//                 expires,
//             } => todo!(),
//             sg721::ExecuteMsg::Revoke { spender, token_id } => todo!(),
//             sg721::ExecuteMsg::ApproveAll { operator, expires } => todo!(),
//             sg721::ExecuteMsg::RevokeAll { operator } => todo!(),
//             sg721::ExecuteMsg::Mint(_) => todo!(),
//             sg721::ExecuteMsg::Burn { token_id } => todo!(),
//             sg721::ExecuteMsg::UpdateCollectionInfo { collection_info } => todo!(),
//             sg721::ExecuteMsg::UpdateTradingStartTime(_) => todo!(),
//             sg721::ExecuteMsg::FreezeCollectionInfo => todo!(),
//         }
//     }
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Minter {},
    CollectionInfo {},
}

impl From<QueryMsg> for Cw721QueryMsg {
    fn from(msg: QueryMsg) -> Cw721QueryMsg {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Cw721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Cw721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Cw721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => Cw721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Cw721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Cw721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfoResponse {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub trading_start_time: Option<Timestamp>,
    pub royalty_info: Option<RoyaltyInfoResponse>,
}
