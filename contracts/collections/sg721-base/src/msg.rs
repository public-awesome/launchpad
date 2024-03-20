use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{
    coin, Addr, BankMsg, Binary, Empty, Event, StdError, StdResult, Timestamp, Uint128,
};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, NftInfoResponse,
    NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::msg::MinterResponse;
use cw721_base::msg::QueryMsg as Cw721QueryMsg;
use cw_ownable::cw_ownable_execute;
use cw_ownable::cw_ownable_query;
use cw_utils::Expiration;
use sg721::RoyaltyInfoResponse;
use sg_std::{Response, SubMsg, NATIVE_DENOM};

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg<T, E> {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },

    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: T,
    },

    /// Burn an NFT the sender has access to
    Burn { token_id: String },

    /// Extension msg
    Extension { msg: E },
}

#[cw_ownable_query]
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(NumTokensResponse)]
    NumTokens {},
    #[returns(ContractInfoResponse)]
    ContractInfo {},
    #[returns(NftInfoResponse<Empty>)]
    NftInfo { token_id: String },
    #[returns(AllNftInfoResponse<Empty>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(MinterResponse)]
    Minter {},
    #[returns(CollectionInfoResponse)]
    CollectionInfo {},
}

impl From<QueryMsg> for Cw721QueryMsg<Empty> {
    fn from(msg: QueryMsg) -> Cw721QueryMsg<Empty> {
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
            QueryMsg::Ownership {} => Cw721QueryMsg::Ownership {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

#[cw_serde]
pub struct CollectionInfoResponse {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<RoyaltyInfoResponse>,
}

impl CollectionInfoResponse {
    pub fn royalty_payout(
        &self,
        collection: Addr,
        payment: Uint128,
        protocol_fee: Uint128,
        finders_fee: Option<Uint128>,
        res: &mut Response,
    ) -> StdResult<Uint128> {
        if let Some(royalty_info) = self.royalty_info.as_ref() {
            if royalty_info.share.is_zero() {
                return Ok(Uint128::zero());
            }
            let royalty = coin((payment * royalty_info.share).u128(), NATIVE_DENOM);
            if payment < (protocol_fee + finders_fee.unwrap_or(Uint128::zero()) + royalty.amount) {
                return Err(StdError::generic_err("Fees exceed payment"));
            }
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: royalty_info.payment_address.to_string(),
                amount: vec![royalty.clone()],
            }));

            let event = Event::new("royalty-payout")
                .add_attribute("collection", collection.to_string())
                .add_attribute("amount", royalty.to_string())
                .add_attribute("recipient", royalty_info.payment_address.to_string());
            res.events.push(event);

            Ok(royalty.amount)
        } else {
            Ok(Uint128::zero())
        }
    }
}

#[cw_serde]
pub enum NftParams<T> {
    NftData {
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: T,
    },
}
