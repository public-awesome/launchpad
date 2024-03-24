use cw721::msg::RoyaltyInfoResponse;
use cw721::traits::Cw721CustomMsg;
use cw721::traits::Cw721State;

use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{coin, Addr, BankMsg, Event, StdError, StdResult, Timestamp, Uint128};
use cw721_base::{
    msg::{
        AllInfoResponse, AllNftInfoResponse, ApprovalResponse, ApprovalsResponse,
        CollectionInfoAndExtensionResponse, MinterResponse, NftInfoResponse, NumTokensResponse,
        OperatorResponse, OperatorsResponse, OwnerOfResponse, QueryMsg as Cw721QueryMsg,
        TokensResponse,
    },
    state::CollectionExtensionAttributes,
};
use cw_ownable::Ownership;
use sg_std::{Response, SubMsg, NATIVE_DENOM};

#[derive(QueryResponses)]
#[cw_serde]
#[allow(deprecated)]
pub enum QueryMsg<
    // Return type of NFT metadata defined in `NftInfo` and `AllNftInfo`.
    TNftExtension,
    // Return type of collection extension defined in `GetCollectionInfo`.
    TCollectionExtension,
    // Custom query msg for custom contract logic. Default implementation returns an empty binary.
    TExtensionQueryMsg,
> {
    #[allow(deprecated)]
    #[returns(CollectionInfoResponse)]
    #[deprecated = "Please use GetCollectionInfo instead"]
    CollectionInfo {},

    // ---- cw721 v0.19.0 msgs ----
    /// Return the owner of the given token, error if token does not exist
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    /// Return operator that can access all of the owner's tokens.
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    /// Return approvals that a token has
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    /// Return approval of a given operator for all tokens of an owner, error if not set
    #[returns(OperatorResponse)]
    Operator {
        owner: String,
        operator: String,
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Total number of tokens issued
    #[returns(NumTokensResponse)]
    NumTokens {},

    #[deprecated(
        since = "0.19.0",
        note = "Please use GetCollectionInfoAndExtension instead"
    )]
    #[returns(CollectionInfoAndExtensionResponse<TCollectionExtension>)]
    /// Deprecated: use GetCollectionInfoAndExtension instead! Will be removed in next release!
    ContractInfo {},

    /// Returns `CollectionInfoAndExtensionResponse`
    #[returns(CollectionInfoAndExtensionResponse<TCollectionExtension>)]
    GetCollectionInfoAndExtension {},

    /// returns `AllInfoResponse` which contains contract, collection and nft details
    #[returns(AllInfoResponse)]
    GetAllInfo {},

    /// Returns `CollectionExtensionAttributes`
    #[returns(CollectionExtensionAttributes)]
    GetCollectionExtensionAttributes {},

    #[deprecated(since = "0.19.0", note = "Please use GetMinterOwnership instead")]
    #[returns(Ownership<Addr>)]
    /// Deprecated: use GetMinterOwnership instead! Will be removed in next release!
    Ownership {},

    /// Return the minter
    #[deprecated(since = "0.19.0", note = "Please use GetMinterOwnership instead")]
    #[returns(MinterResponse)]
    /// Deprecated: use GetMinterOwnership instead! Will be removed in next release!
    Minter {},

    #[returns(Ownership<Addr>)]
    GetMinterOwnership {},

    #[returns(Ownership<Addr>)]
    GetCreatorOwnership {},

    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract
    #[returns(NftInfoResponse<TNftExtension>)]
    NftInfo { token_id: String },

    #[returns(Option<NftInfoResponse<TNftExtension>>)]
    GetNftByExtension {
        extension: TNftExtension,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients
    #[returns(AllNftInfoResponse<TNftExtension>)]
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Custom msg query. Default implementation returns an empty binary.
    #[returns(())]
    Extension { msg: TExtensionQueryMsg },

    #[returns(())]
    GetCollectionExtension { msg: TCollectionExtension },

    #[returns(Option<String>)]
    GetWithdrawAddress {},
}

impl<TNftExtension, TCollectionExtension, TExtensionQueryMsg>
    From<QueryMsg<TNftExtension, TCollectionExtension, TExtensionQueryMsg>>
    for Cw721QueryMsg<TNftExtension, TCollectionExtension, TExtensionQueryMsg>
where
    TNftExtension: Cw721State,
    TCollectionExtension: Cw721State,
    TExtensionQueryMsg: Cw721CustomMsg,
{
    #[allow(deprecated)]
    fn from(
        msg: QueryMsg<TNftExtension, TCollectionExtension, TExtensionQueryMsg>,
    ) -> Cw721QueryMsg<TNftExtension, TCollectionExtension, TExtensionQueryMsg> {
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
            QueryMsg::GetCollectionInfoAndExtension {} => {
                Cw721QueryMsg::GetCollectionInfoAndExtension {}
            }
            QueryMsg::Ownership {} => Cw721QueryMsg::Ownership {},
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            QueryMsg::GetMinterOwnership {} => Cw721QueryMsg::GetMinterOwnership {},
            QueryMsg::GetCreatorOwnership {} => Cw721QueryMsg::GetCreatorOwnership {},
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
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

#[cw_serde]
#[deprecated = "Please use `CollectionInfo<CollectionInfoExtension<RoyaltyInfo>>` instead"]
pub struct CollectionInfoResponse {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<RoyaltyInfoResponse>,
}

#[allow(deprecated)]
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
