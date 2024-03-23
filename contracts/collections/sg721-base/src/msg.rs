use cw721::msg::RoyaltyInfoResponse;
use cw721::traits::Cw721State;

use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{coin, Addr, BankMsg, Event, StdError, StdResult, Timestamp, Uint128};
use cw721_base::{
    msg::{
        AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, MinterResponse, NftInfoResponse,
        NumTokensResponse, OperatorsResponse, OwnerOfResponse, QueryMsg as Cw721QueryMsg,
        TokensResponse,
    },
    state::CollectionMetadataAndExtension,
    DefaultOptionCollectionMetadataExtensionMsg,
};
use cw_ownable::Ownership;
use sg_std::{Response, SubMsg, NATIVE_DENOM};

#[derive(QueryResponses)]
#[cw_serde]
#[allow(deprecated)]
pub enum QueryMsg<
    // Return type of NFT metadata defined in `NftInfo` and `AllNftInfo`.
    TNftMetadataExtension,
    // Return type of collection metadata extension defined in `GetCollectionMetadata`.
    TCollectionMetadataExtension,
> {
    #[allow(deprecated)]
    #[returns(CollectionInfoResponse)]
    #[deprecated = "Please use GetCollectionMetadata instead"]
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

    #[deprecated(since = "0.19.0", note = "Please use GetCollectionMetadata instead")]
    #[returns(CollectionMetadataAndExtension<DefaultOptionCollectionMetadataExtensionMsg>)]
    /// Deprecated: use GetCollectionMetadata instead! Will be removed in next release!
    ContractInfo {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract
    #[returns(CollectionMetadataAndExtension<TCollectionMetadataExtension>)]
    GetCollectionMetadata {},

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
    #[returns(NftInfoResponse<TNftMetadataExtension>)]
    NftInfo { token_id: String },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients
    #[returns(AllNftInfoResponse<TNftMetadataExtension>)]
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

    #[returns(Option<String>)]
    GetWithdrawAddress {},

    // -- below queries, Extension and GetCollectionMetadataExtension, are just dummies, since type annotations are required for
    // -- TNftMetadataExtension and TCollectionMetadataExtension, Error:
    // -- "type annotations needed: cannot infer type for type parameter `TNftMetadataExtension` declared on the enum `Cw721QueryMsg`"
    /// Use NftInfo instead.
    /// No-op / NFT metadata query returning empty binary, needed for inferring type parameter during compile.
    ///
    /// Note: it may be extended in case there are use cases e.g. for specific NFT metadata query.
    #[returns(())]
    #[deprecated(since = "0.19.0", note = "Please use GetNftMetadata instead")]
    Extension { msg: TNftMetadataExtension },

    #[returns(())]
    GetNftMetadata { msg: TNftMetadataExtension },

    /// Use GetCollectionMetadata instead.
    /// No-op / collection metadata extension query returning empty binary, needed for inferring type parameter during compile
    ///
    /// Note: it may be extended in case there are use cases e.g. for specific collection metadata query.
    #[returns(())]
    GetCollectionMetadataExtension { msg: TCollectionMetadataExtension },
}

impl<TNftMetadataExtension, TCollectionMetadataExtension>
    From<QueryMsg<TNftMetadataExtension, TCollectionMetadataExtension>>
    for Cw721QueryMsg<TNftMetadataExtension, TCollectionMetadataExtension>
where
    TNftMetadataExtension: Cw721State,
    TCollectionMetadataExtension: Cw721State,
{
    #[allow(deprecated)]
    fn from(
        msg: QueryMsg<TNftMetadataExtension, TCollectionMetadataExtension>,
    ) -> Cw721QueryMsg<TNftMetadataExtension, TCollectionMetadataExtension> {
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
            QueryMsg::GetCollectionMetadata {} => Cw721QueryMsg::GetCollectionMetadata {},
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
#[deprecated = "Please use `CollectionMetadata<CollectionMetadataExtension<RoyaltyInfo>>` instead"]
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
