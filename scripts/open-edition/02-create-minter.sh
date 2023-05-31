KEY=$(starsd keys show $ADMIN | jq -r .name)
FACTORY=stars1a90ylq0yfvgg4y84x9vwk7n7fjl2lhkwa88vt9tr9fvcwlh7g7sqdh5sdc
SG721_CODE_ID=2354

# pub enum NftMetadataType {
#     OnChainMetadata,
#     OffChainMetadata,
# }
# pub struct NftData {
#     pub nft_data_type: NftMetadataType,
#     pub extension: Option<Metadata>,
#     pub token_uri: Option<String>,
# }

# init msg
# OpenEditionVendingMinterInitMsgExtension {
#     pub admin: Addr,
#     pub payment_address: Option<Addr>,
#     pub nft_data: NftData,
#     pub start_time: Timestamp,
#     pub end_time: Timestamp,
#     pub per_address_limit: u32,
# }
# collection params
# CollectionParams {
#     /// The collection code id
#     pub code_id: u64,
#     pub name: String,
#     pub symbol: String,
#     pub info: CollectionInfo<RoyaltyInfoResponse>,
# }
# CollectionInfo {
    # pub creator: String,
    # pub description: String,
    # pub image: String,
    # pub external_link: Option<String>,
    # pub explicit_content: Option<bool>,
    # pub start_trading_time: Option<Timestamp>,
    # pub royalty_info: Option<T>,
# }

# add a few minutes buffer to start time
TIME=$(date -v+60S +%s)
END_TIME=$(date -v+6000S +%s)

MSG=$(cat <<EOF
{
    "create_minter": {
        "init_msg": {
            "nft_data": {
                "nft_data_type": "off_chain_metadata",
                "token_uri": "ipfs://bafybeiey2heysue3px2tgc523cmjbfjlox5zfzzan5syzdooikdvimtxwq/2047"
            },
            "start_time": "$(echo $TIME)000000000",
            "end_time": "$(echo $END_TIME)000000000",
            "mint_price": { "amount": "50000000", "denom": "ustars" },
            "per_address_limit": 30
        },
        "collection_params": {
            "code_id": $SG721_CODE_ID,
            "name": "Test Collection yubo",
            "symbol": "YUBO",
            "info": {
                "creator": "$ADMIN",
                "description": "Test Collection yubo",
                "image": "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/2047.jpg"
            }
        }
    }
}
EOF
)

echo $MSG

starsd tx wasm execute $FACTORY "$MSG" --amount 5000000000ustars \
--gas-prices 0.025ustars --gas 5000000 --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .