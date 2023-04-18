use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cosmwasm_std::Timestamp;
use cw721_base::msg::MintMsg;
use cw_utils::Expiration;
use sg721::{RoyaltyInfoResponse, UpdateCollectionInfoMsg};
use sg721_base::ExecuteMsg as Sg721ExecuteMsg;

#[cw_serde]
pub enum ExecuteMsg<T, E> {
    /// Freeze token metadata so creator can no longer update token uris
    FreezeTokenMetadata {},
    /// Creator calls can update token uris
    UpdateTokenMetadata {
        token_id: String,
        token_uri: Option<String>,
    },
    /// Enable updatable for updating token metadata. One time migration fee for sg721-base to sg721-updatable.
    EnableUpdatable {},
    // Sg721Base msgs
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },
    Burn {
        token_id: String,
    },
    UpdateCollectionInfo {
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    UpdateTradingStartTime(Option<Timestamp>),
    FreezeCollectionInfo {},
    Mint(MintMsg<T>),
    Extension {
        msg: E,
    },
}

impl<T, E> From<ExecuteMsg<T, E>> for Sg721ExecuteMsg
where
    T: Clone + PartialEq + Into<Option<cosmwasm_std::Empty>>,
    Option<cosmwasm_std::Empty>: From<T>,
{
    fn from(msg: ExecuteMsg<T, E>) -> Sg721ExecuteMsg {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Sg721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => Sg721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Sg721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::ApproveAll { operator, expires } => {
                Sg721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::Revoke { spender, token_id } => {
                Sg721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::RevokeAll { operator } => Sg721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Burn { token_id } => Sg721ExecuteMsg::Burn { token_id },
            ExecuteMsg::UpdateCollectionInfo { collection_info } => {
                Sg721ExecuteMsg::UpdateCollectionInfo { collection_info }
            }
            ExecuteMsg::FreezeCollectionInfo {} => Sg721ExecuteMsg::FreezeCollectionInfo {},
            ExecuteMsg::Mint(MintMsg {
                token_id,
                owner,
                token_uri,
                extension,
            }) => Sg721ExecuteMsg::Mint(MintMsg {
                token_id,
                owner,
                token_uri,
                extension: extension.into(),
            }),
            _ => unreachable!("Invalid ExecuteMsg"),
        }
    }
}
